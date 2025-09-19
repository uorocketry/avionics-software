fn main() {
	// Needed by embedded_test crate to function
	println!("cargo::rustc-link-arg=-Tembedded-test.x");
}
