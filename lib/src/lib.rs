use anyhow::Result;
use std::{collections::HashMap, fs, path::PathBuf, process::Command, str::from_utf8};
use syn::{
	Ident, Item, ItemType,
	Type::{self, Tuple},
	TypeTuple,
};

/// Tyep alias for a Migration. This is a String.
pub type Migration = String;

/// Looking for files containing: `pub type Migrations`
/// This command relies on having `git` installed and the
/// passed `repo` being a valid git repository.
pub fn get_files(repo: &PathBuf) -> Result<Vec<PathBuf>> {
	const PATTERN: &str = "pub type Migrations";

	let mut grep = Command::new("git");
	grep.args(["grep", "--name-only", PATTERN]);
	let output = grep.current_dir(repo).output();

	match output {
		Ok(o) => {
			let arr: Vec<PathBuf> = o
				.stdout
				.split(|c| c == &10)
				.map(|a| from_utf8(a).unwrap())
				.filter(|s| !s.is_empty())
				.map(|s| PathBuf::from(repo).join(PathBuf::from(s)))
				.collect();
			Ok(arr)
		}
		Err(e) => {
			eprint!("{repo:?}");
			eprint!("{e:?}");
			todo!()
		}
	}
}

/// Get one migration from a Type item.
pub fn get_migration(e: &Type) -> Option<String> {
	match e {
		syn::Type::Path(p) => {
			let segment = p.path.segments.iter().nth(1).unwrap();
			let xx = &(segment.ident.clone() as Ident);
			Some(xx.to_string())
		}
		_ => None,
	}
}

/// Extract all Migration from the elements of a Tuppe
pub fn string_from_tuple(tuple: &TypeTuple) -> Vec<Migration> {
	tuple.elems.iter().map(|e| get_migration(e).unwrap()).collect::<Vec<Migration>>()
}

/// Get all Migrations
pub fn get_migrations(it: &ItemType) -> Result<Vec<Migration>> {
	let rr: Vec<String> = match &*it.ty {
		Tuple(t) => string_from_tuple(t),
		_ => unreachable!(),
	};

	Ok(rr)
}

/// Find all Migrations for a given repo
pub fn find(repo: &PathBuf) -> Result<HashMap<PathBuf, Vec<Migration>>> {
	let files = get_files(repo)?;
	let mut res: HashMap<PathBuf, Vec<Migration>> = HashMap::new();

	for file in files {
		let code = fs::read_to_string(&file)?;
		let syntax = syn::parse_file(&code)?;

		let hits: Vec<&Item> =
			syntax.items.iter().filter(|&item| matches!(item, syn::Item::Type(i) if i.ident == "Migrations")).collect();

		assert!(hits.len() == 1);
		let hit = hits.first().unwrap();

		let result: Vec<String> = match hit {
			syn::Item::Type(it) => get_migrations(it).unwrap(),
			_ => vec![],
		};
		res.insert(file, result);
	}
	Ok(res)
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::env;

	fn setup() {
		let polkadot_repo: &str = &env::var("POLKADOT_REPO").unwrap_or_default();
		if polkadot_repo.is_empty() {
			env::set_var("POLKADOT_REPO", "/projects/polkadot");
		}

		let cumulus_repo: &str = &env::var("CUMULUS_REPO").unwrap_or_default();
		if cumulus_repo.is_empty() {
			env::set_var("CUMULUS_REPO", "/projects/cumulus");
		}
	}

	#[test]
	fn it_find_files() {
		setup();
		let polkadot_repo: &str = &env::var("POLKADOT_REPO").unwrap();
		let result = get_files(&PathBuf::from(polkadot_repo)).unwrap();
		assert!(result.len() == 4);
	}

	#[test]
	fn it_finds_migrations_polkadot() {
		setup();
		let polkadot_repo: &str = &env::var("POLKADOT_REPO").unwrap();
		let result = find(&PathBuf::from(polkadot_repo)).unwrap();
		assert!(result.len() == 4);
		println!("result = {:?}", result);
	}

	#[test]
	#[ignore = "Migration were not updated in Cumulus yet"]
	fn it_finds_migrations_cumulus() {
		setup();
		let cumulus_repo: &str = &env::var("CUMULUS_REPO").unwrap();
		let result = find(&PathBuf::from(cumulus_repo)).unwrap();
		assert!(result.is_empty()); // The migration fix was not done yet
		println!("result = {:?}", result);
	}
}
