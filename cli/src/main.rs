mod opts;

use clap::{crate_name, crate_version, Parser};
use env_logger::Env;
use log::{debug, info};
use opts::*;
use std::string::String;

fn main() -> Result<(), String> {
	env_logger::Builder::from_env(Env::default().default_filter_or("none")).init();
	info!("Running {} v{}", crate_name!(), crate_version!());

	let opts: Opts = Opts::parse();
	debug!("opts:\n{:#?}", opts);

	let mut overall_valid = true;

	match opts.subcmd {
		SubCommand::List(list_opts) => {
			debug!("list_opts:\n{:#?}", list_opts);
			let migrations_map = submig_lib::find(&list_opts.repo);
			println!("Checking migrations in repo: {}", list_opts.repo.display());
			match migrations_map {
				Ok(hmap) => {
					for (file, (valid, invalid)) in hmap {
						if let Some(pattern) = &list_opts.pattern {
							if file.display().to_string().contains(pattern) {
								println!("{}:", file.display());
								for migration in &valid {
									println!("  - ✅ {migration}");
								}

								if !invalid.is_empty() {
									overall_valid &= false
								};
								for migration in invalid {
									println!("  - ❌ {migration}");
								}
							}
						} else {
							println!("{}:", file.display());
							for migration in &valid {
								println!("  - ✅ {migration}");
							}

							if !invalid.is_empty() {
								overall_valid &= false
							};
							for migration in invalid {
								println!("  - ❌ {migration}");
							}
						}
					}
					println!();
				}
				Err(e) => eprint!("An error occured: {e:?}"),
			}
		}
	}

	if overall_valid {
		Ok(())
	} else {
		Err(String::from("Some migrations are invalid"))
	}
}
