use anyhow::{bail, Result};
use std::{fs, path::PathBuf, process::Command, str::from_utf8};
use syn::{
	Ident, Item, ItemType,
	Type::{self, Tuple},
	TypeTuple,
};

/// Looking for files containing: `pub type Migrations`
pub fn get_files(repo: &PathBuf) -> Result<Vec<PathBuf>> {
	let mut grep = Command::new("git");
	grep.args(["grep", "--name-only", "pub type Migrations"]);
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
		Err(_) => todo!(),
	}
}

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

pub fn string_from_tuple(tuple: &TypeTuple) -> Vec<String> {
	let oo: Vec<String> = tuple.elems.iter().map(|e| get_migration(e).unwrap()).collect();
	oo
}

pub fn get_migrations(it: &ItemType) -> Result<Vec<String>> {
	let rr: Vec<String> = match &*it.ty {
		Tuple(t) => string_from_tuple(t),
		_ => unreachable!(),
	};

	Ok(rr)
}

pub fn find(repo: &PathBuf) -> Result<Vec<String>> {
	let files = get_files(repo)?;

	for file in files {
		println!("processing {file:?}");
		let code = fs::read_to_string(&file)?;
		let syntax = syn::parse_file(&code)?;

		let hits: Vec<&Item> = syntax
			.items
			.iter()
			.filter(|&item| match item {
				syn::Item::Type(i) if i.ident == "Migrations" => true,
				_ => false,
			})
			.collect();

		assert!(hits.len() == 1);
		let hit = hits.first().unwrap();

		let result: Vec<String> = match hit {
			syn::Item::Type(it) => get_migrations(it).unwrap(),
			_ => vec![],
		};

		return Ok(result);
	}
	bail!("meh");
}

#[cfg(test)]
mod tests {
	use super::*;
	const PROJECT_DIR_POLKADOT: &str = "/projects/polkadot";
	const PROJECT_DIR_CUMULUS: &str = "/projects/cumulus";

	#[test]
	fn it_find_files() {
		let result = get_files(&PathBuf::from(PROJECT_DIR_POLKADOT)).unwrap();
		assert!(result.len() == 4);
	}

	#[test]
	fn it_finds_migrations_polkadot() {
		let result = find(&PathBuf::from(PROJECT_DIR_POLKADOT)).unwrap();
		assert!(result.len() == 4);
		println!("result = {:?}", result);
	}

	#[test]
	fn it_finds_migrations_cumulus() {
		let result = find(&PathBuf::from(PROJECT_DIR_CUMULUS)).unwrap();
		assert!(result.len() == 0); // The migration fix was not done yet
		println!("result = {:?}", result);
	}
}
