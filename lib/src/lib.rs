mod error;
pub use error::*;

use log::debug;
use regex::Regex;
use std::{collections::HashMap, fs, path::PathBuf, process::Command, str::from_utf8, fmt::Display};
use syn::{
	Ident, Item, ItemType,
	Type::{self, Tuple},
	TypeTuple,
};

/// Tyep alias for a Migration. This is a String.
#[derive(Debug, Clone)]
pub enum Migration {
	Ok(String),
	NotOk(),
}

impl Display for Migration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Migration::Ok(s) => s.to_string(),
            Migration::NotOk() => "err".to_string(),
        };
		f.write_str(&s)
    }
}

/// Looking for files containing: `pub type Migrations`
/// This command relies on having `git` installed and the
/// passed `repo` being a valid git repository.
pub fn get_files(repo: &PathBuf, folder: &PathBuf) -> Result<Vec<PathBuf>> {
	const PATTERN: &str = "pub type Migrations";

	let mut grep = Command::new("git");
	grep.args(["grep", "--name-only", PATTERN]);
	let output = grep.current_dir(repo.join(folder)).output();

	match output {
		Ok(o) => {
			let arr: Vec<PathBuf> = o
				.stdout
				.split(|c| c == &10)
				.map(|a| from_utf8(a).unwrap())
				.filter(|s| !s.is_empty())
				.map(|s| PathBuf::from(repo).join(folder).join(PathBuf::from(s)))
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
fn get_migration(e: &Type) -> Result<Option<Migration>> {
	match e {
		syn::Type::Path(p) => {
			let segment = p.path.segments.iter().nth(1).ok_or(SubmigError::NonStandard)?;
			let ident = &(segment.ident.clone() as Ident);
			Ok(Some(Migration::Ok(ident.to_string())))
		}
		_ => Err(SubmigError::NonStandard),
	}
}

/// Extract all Migration from the elements of a Tuppe
fn string_from_tuple(tuple: &TypeTuple) -> Result<Vec<Migration>> {
	log::debug!("tuple: {tuple:?}");

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


// fn string_from_path(p: &syn::TypePath) -> Vec<String> {
// 	log::debug!("path: {p:?}");
//     vec!["todo".into()]
// }

/// Get all Migrations
fn get_migrations(it: &ItemType) -> Result<Vec<Migration>> {
	let migrations: Vec<Migration> = match &*it.ty {
		Tuple(t) => string_from_tuple(t)?,
		// Type::Path(p) => {
		// 	log::info!("Path: {p:?}");
		// 	// get_migrations(p.qself).unwrap_or_default()
		// 	string_from_path(p)
		// }
		// x => {

		// 	log::error!("Unexpected type/format: {x:?}");
		// 	unreachable!()
		// }
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
		migrations.iter().filter(|m| m.to_string() == "Unreleased" || version_regexp.is_match(&m.to_string())).map(|s| s.clone()).collect();
	let invalid = migrations
		.iter()
		.filter(|m| m.to_string() != "Unreleased" && !version_regexp.is_match(&m.to_string()))
		.map(|s| s.clone())
		.collect();
	(valid, invalid)
}

type SearchResult = HashMap<PathBuf, (Vec<Migration>, Vec<Migration>)>;

/// Find all Migrations for a given repo
/// It returns a Hashmap per file with a tuple made of the Vec of valid and invalid migrations
/// based on the naming.
pub fn find(repo: &PathBuf, folder: &PathBuf) -> Result<SearchResult> {
	let files = get_files(repo, folder)?;
	let mut res: SearchResult = HashMap::new();

	for file in files {
		let code = fs::read_to_string(&file).map_err(|_e| SubmigError::IO)?;
		let syntax = syn::parse_file(&code).map_err(|_| SubmigError::Parsing)?;

		let hits: Vec<&Item> =
			syntax.items.iter().filter(|&item| matches!(item, syn::Item::Type(i) if i.ident == "Migrations")).collect();

		debug!("Found {} Migration hits in {}", hits.len(), file.display());
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
		let polkadot_repo: &str = &env::var("REPO_POLKADOT").unwrap_or_default();
		if polkadot_repo.is_empty() {
			env::set_var("REPO_POLKADOT", "/projects/polkadot-sdk");
		}

		let cumulus_repo: &str = &env::var("REPO_CUMULUS").unwrap_or_default();
		if cumulus_repo.is_empty() {
			env::set_var("REPO_CUMULUS", "/projects/polkadot-sdk");
		}
	}

	#[test]
	fn it_find_files() {
		setup();
		let polkadot_repo: &str = &env::var("REPO_POLKADOT").unwrap();
		let result = get_files(&PathBuf::from(polkadot_repo), &PathBuf::from("polkadot")).unwrap();
		assert!(result.len() == 4);
	}

	#[test]
	fn it_finds_migrations_polkadot() {
		setup();
		let polkadot_repo: &str = &env::var("REPO_POLKADOT").unwrap();
		let result = find(&PathBuf::from(polkadot_repo), &PathBuf::from("polkadot")).unwrap();
		assert!(result.len() == 4);
		println!("result = {:?}", result);
	}

	#[test]
	#[ignore = "Migration were not updated in Cumulus yet"]
	fn it_finds_migrations_cumulus() {
		setup();
		let cumulus_repo: &str = &env::var("REPO_CUMULUS").unwrap();
		let result = find(&PathBuf::from(cumulus_repo), &PathBuf::from("cumulus")).unwrap();
		assert!(result.is_empty()); // The migration fix was not done yet
		println!("result = {:?}", result);
	}
}
