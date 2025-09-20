use glob::glob;
use prost_build::Config;
use std::io::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
	let mut config = Config::new();
	config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
	config.btree_map(&["."]);

	// Iterate through all .proto files in the src directory and its subdirectories
	let mut protos_paths: Vec<PathBuf> = Vec::new();
	for entry in glob("src/**/*.proto").expect("Failed to read glob pattern") {
		match entry {
			Ok(path) => protos_paths.push(path),
			Err(error) => eprintln!("glob error: {error}"),
		}
	}

	config.compile_protos(&protos_paths, &["src/"])?;

	Ok(())
}
