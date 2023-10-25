mod error;
pub use error::*;

use log::debug;
use regex::Regex;
use std::{collections::HashMap, fmt::Display, fs, path::PathBuf, process::Command, str::from_utf8};
use syn::{
	Ident, Item,
	Type::{self, Path, Tuple},
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

impl From<&str> for Migration {
	fn from(s: &str) -> Self {
		Migration::Ok(s.to_string())
	}
}

/// Looking for files containing: `pub type Migrations`
/// This command relies on having `git` installed and the
/// passed `repo` being a valid git repository.
pub fn get_files(
	repo: &PathBuf,
	// folder: &PathBuf
) -> Result<Vec<PathBuf>> {
	const PATTERN: &str = "pub type Migrations";

	let mut grep = Command::new("git");
	grep.args(["grep", "--name-only", PATTERN]);
	// let output = grep.current_dir(repo.join(folder)).output();
	let output = grep.current_dir(repo).output();

	match output {
		Ok(o) => {
			let arr: Vec<PathBuf> = o
				.stdout
				.split(|c| c == &10)
				.map(|a| from_utf8(a).unwrap())
				.filter(|s| !s.is_empty())
				// .map(|s| PathBuf::from(repo).join(folder).join(PathBuf::from(s)))
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
/// It returns a Vec of Migrations, assuming they follow the expected naming
fn get_migration(t: &Type) -> Result<Vec<Migration>> {
	match t {
		Path(p) => {
			// If the naming changes and we need to hanle the full segment:
			// let segment: String = p.path.segments.iter().map(|s| {
			// 	s.ident.to_string()
			// }).collect();
			// // todo: handle separators
			// Ok(vec![Migration::Ok(segment)])

			let segment = p.path.segments.iter().nth(1).ok_or(SubmigError::NonStandard)?;
			let ident = &(segment.ident.clone() as Ident);
			Ok(vec![(Migration::Ok(ident.to_string()))])
		}
		Tuple(t) => {
			log::debug!("tuple: nb elems: {}", t.elems.len());

			let content = t.elems.iter().flat_map(get_migration).flatten().collect();

			Ok(content)
		}
		Type::Paren(p) => {
			log::debug!("{p:?}");
			let content = p.elem.clone();
			get_migration(content.as_ref())
		}
		x => {
			log::warn!("Non standard: {x:?})");
			Err(SubmigError::NonStandard)
		}
	}
}
/// Get all Migrations
fn get_migrations(it: &Item) -> Result<Vec<Migration>> {
	log::debug!("get_migrations");

	let migrations: Vec<Migration> = match it {
		Item::Type(t) => get_migration(&t.ty).unwrap(),
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

	let valid = migrations
		.iter()
		.filter(|m| m.to_string() == "Unreleased" || version_regexp.is_match(&m.to_string()))
		.cloned()
		.collect();
	let invalid = migrations
		.iter()
		.filter(|m| m.to_string() != "Unreleased" && !version_regexp.is_match(&m.to_string()))
		.cloned()
		.collect();
	(valid, invalid)
}

type SearchResult = HashMap<PathBuf, (Vec<Migration>, Vec<Migration>)>;

/// Find all Migrations for a given repo
/// It returns a Hashmap per file with a tuple made of the Vec of valid and invalid migrations
/// based on the naming.
pub fn find(
	repo: &PathBuf,
	// folder: &PathBuf
) -> Result<SearchResult> {
	let files = get_files(
		repo,
		// folder
	)?;
	let mut res: SearchResult = HashMap::new();

	for file in files {
		let code = fs::read_to_string(&file).map_err(|_e| SubmigError::IO)?;
		let syntax = syn::parse_file(&code).map_err(|_| SubmigError::Parsing)?;

		let hits: Vec<&Item> =
			syntax.items.iter().filter(|&item| matches!(item, syn::Item::Type(i) if i.ident == "Migrations")).collect();

		debug!("Found {} Migration hits in {}", hits.len(), file.display());
		if let Some(hit) = hits.first() {
			let migrations: Vec<Migration> = get_migrations(hit)?;
			let (valid, invalid) = check_naming(migrations);
			res.insert(file, (valid, invalid));
		}
	}
	Ok(res)
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::env;

	fn setup() {
		let repo_polkadot_sdk: &str = &env::var("REPO_POLKADOT_SDK").unwrap_or_default();
		if repo_polkadot_sdk.is_empty() {
			env::set_var("REPO_POLKADOT_SDK", "/projects/polkadot-sdk");
		}

		let repo_fellowship_runtimes: &str = &env::var("REPO_FELLOWSHIP_RUNTIMES").unwrap_or_default();
		if repo_fellowship_runtimes.is_empty() {
			env::set_var("REPO_FELLOWSHIP_RUNTIMES", "/projects/fellowship-runtimes");
		}
	}

	#[test]
	fn it_find_files() {
		setup();
		let polkadot_repo: &str = &env::var("REPO_POLKADOT_SDK").unwrap();
		let result = get_files(&PathBuf::from(polkadot_repo)).unwrap();
		assert_eq!(12, result.len());
	}

	#[test]
	fn it_finds_migrations_polkadot_sdk() {
		setup();
		let polkadot_repo: &str = &env::var("REPO_POLKADOT_SDK").unwrap();
		let result = find(&PathBuf::from(polkadot_repo)).unwrap();
		assert_eq!(11, result.len());
		println!("result = {:?}", result);
	}

	#[test]
	fn it_finds_migrations_fellowship_runtimes() {
		setup();
		let polkadot_repo: &str = &env::var("REPO_FELLOWSHIP_RUNTIMES").unwrap();
		let result = find(&PathBuf::from(polkadot_repo)).unwrap();
		assert_eq!(6, result.len());
		println!("result = {:?}", result);
	}
}
