mod error;
pub use error::*;

use log::debug;
use regex::Regex;
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
			eprintln!("repo: {repo:?}");
			eprintln!("{e:?}");
			todo!()
		}
	}
}

/// Get one migration from a Type item.
fn get_migration(e: &Type) -> Result<Option<String>> {
	match e {
		syn::Type::Path(p) => {
			let segment = p.path.segments.iter().nth(1).ok_or(SubmigError::NonStandard)?;
			let ident = &(segment.ident.clone() as Ident);
			Ok(Some(ident.to_string()))
		}
		_ => Err(SubmigError::NonStandard),
	}
}

/// Extract all Migration from the elements of a Tuppe
fn string_from_tuple(tuple: &TypeTuple) -> Result<Vec<Migration>> {
	let mig = tuple.elems.iter().map(|e| match get_migration(e) {
		Ok(m) => Ok(m),
		Err(e) => Err(e),
	});
	let no_error = mig.clone().map(|i| i.is_ok()).all(|x| x);
	if no_error {
		let vec = mig.map(|i| i.unwrap().unwrap()).collect::<Vec<Migration>>();
		Ok(vec)
	} else {
		Err(SubmigError::NonStandard)
	}
}

/// Get all Migrations
fn get_migrations(it: &ItemType) -> Result<Vec<Migration>> {
	let migrations: Vec<String> = match &*it.ty {
		Tuple(t) => string_from_tuple(t)?,
		_ => unreachable!(),
	};
	debug!("Migrations: {migrations:?}");
	Ok(migrations)
}

/// We expect all migrations to be either:
/// `VXXXX` or `Unreleased`.
/// Returns the valid and invalid migrations.
fn check_naming(migrations: Vec<Migration>) -> (Vec<Migration>, Vec<Migration>) {
	let version_regexp = Regex::new(r"^V\d{4}$").unwrap();

	let valid =
		migrations.iter().filter(|m| m == &"Unreleased" || version_regexp.is_match(m)).map(|s| s.to_string()).collect();
	let invalid = migrations
		.iter()
		.filter(|m| m != &"Unreleased" && !version_regexp.is_match(m))
		.map(|s| s.to_string())
		.collect();
	(valid, invalid)
}

type SearchResult = HashMap<PathBuf, (Vec<Migration>, Vec<Migration>)>;

/// Find all Migrations for a given repo
/// It returns a Hashmap per file with a tuple made of the Vec of valid and invalid migrations
/// based on the naming.
pub fn find(repo: &PathBuf) -> Result<SearchResult> {
	let files = get_files(repo)?;
	let mut res: SearchResult = HashMap::new();

	for file in files {
		let code = fs::read_to_string(&file).map_err(|_e| SubmigError::IO)?;
		let syntax = syn::parse_file(&code).map_err(|_| SubmigError::Parsing)?;

		let hits: Vec<&Item> =
			syntax.items.iter().filter(|&item| matches!(item, syn::Item::Type(i) if i.ident == "Migrations")).collect();

		debug!("Found {} Migration hits", hits.len());
		assert!(hits.len() == 1);
		let hit = hits.first().unwrap();

		let migrations: Vec<Migration> = match hit {
			syn::Item::Type(it) => get_migrations(it)?,
			_ => vec![],
		};

		let (valid, invalid) = check_naming(migrations);

		res.insert(file, (valid, invalid));
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
