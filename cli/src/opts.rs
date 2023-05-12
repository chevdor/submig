use clap::{crate_authors, crate_version, Parser, Subcommand};
use std::env;
use std::path::PathBuf;

/// Experiemtnal command line utility to find polkadot Migrations
#[derive(Debug, Parser)]
#[clap(version = crate_version!(), author = crate_authors!())]
pub struct Opts {
	#[clap(subcommand)]
	pub subcmd: Option<SubCommand>,
}

/// You can find all available commands below.
#[derive(Subcommand, Debug)]
pub enum SubCommand {
	#[clap(version = crate_version!(), author = crate_authors!())]
	List(ListOpts),
}

/// Find and list the migrations that have been found
#[derive(Parser, Debug)]
pub struct ListOpts {
	/// The path of your repo
	#[clap(index = 1, env = "REPO_POLKADOT")]
	pub repo: PathBuf,

	/// Optional pattern to filter output
	#[clap(long, short, alias = "grep")]
	pub pattern: Option<String>,
}
