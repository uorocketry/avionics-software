use std::io::Result;
use std::path::PathBuf;

use glob::glob;
use prost_build::Config;

fn main() -> Result<()> {
	let mut config = Config::new();
	config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
	config.btree_map(&["."]);

	// Iterate through all .proto files in the proto directory and its subdirectories
	let mut protos_paths: Vec<PathBuf> = Vec::new();
	for entry in glob("proto/**/*.proto").expect("Failed to read glob pattern") {
		match entry {
			Ok(path) => protos_paths.push(path),
			Err(error) => eprintln!("glob error: {error}"),
		}
	}

	let protoc_bin_path = protoc_bin_vendored::protoc_bin_path().expect("Failed to locate vendored protoc binary");
	std::env::set_var("PROTOC", protoc_bin_path);

	let include_dir = protoc_bin_vendored::include_path().expect("Failed to locate vendored protobuf includes");
	let include_dir = include_dir.to_string_lossy().to_string();

	let includes = [include_dir.as_str(), "proto/"];
	config.compile_protos(&protos_paths, &includes)?;

	Ok(())
}
