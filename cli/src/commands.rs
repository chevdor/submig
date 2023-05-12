use std::path::PathBuf;

pub fn list_migrations(repo: &PathBuf, pattern: Option<String>) -> submig_lib::Result<bool> {
	let migrations_map = submig_lib::find(repo);
	println!("Checking migrations in repo: {}", repo.display());
	let mut overall_valid = true;
	match migrations_map {
		Ok(hmap) => {
			for (file, (valid, invalid)) in hmap {
				if let Some(pattern) = &pattern {
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
	Ok(overall_valid)
}
