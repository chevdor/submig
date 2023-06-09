mod commands;
mod opts;

use crate::commands::*;
use clap::{crate_name, crate_version, Parser};
use env_logger::Env;
use log::{debug, info};
use opts::*;
use std::{path::PathBuf, string::String};

fn main() -> Result<(), String> {
	env_logger::Builder::from_env(Env::default().default_filter_or("none")).init();
	info!("Running {} v{}", crate_name!(), crate_version!());

	let opts: Opts = Opts::parse();
	debug!("opts:\n{:#?}", opts);

	let overall_valid = match opts.subcmd {
		// If no command is passed, we use list and hope that the REPO ENV is set
		None => {
			let repo =
				std::env::var("REPO_POLKADOT").expect("If you pass no command, the REPO_POLKADOT ENV must be defined.");
			list_migrations(&PathBuf::from(repo), None).unwrap()
		}

		Some(SubCommand::List(list_opts)) => {
			debug!("list_opts:\n{:#?}", list_opts);
			list_migrations(&list_opts.repo, list_opts.pattern).unwrap()
		}
	};

	if overall_valid {
		Ok(())
	} else {
		Err(String::from("Some migrations are invalid"))
	}
}
