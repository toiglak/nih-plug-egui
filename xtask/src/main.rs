fn main() -> nih_plug_xtask::Result<()> {
	// If bundled/ exists, xtask will not overwrite it on macos showing stale vst.
	let bundled_dir = std::path::Path::new("target/bundled");
	if bundled_dir.exists() {
		if let Err(err) = std::fs::remove_dir_all(bundled_dir) {
			eprintln!("Warning: Failed to clean target/bundled: {err}");
		}
	}

	nih_plug_xtask::main()
}
