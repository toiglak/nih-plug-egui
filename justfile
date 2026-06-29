run:
	cargo +stable run --release --example simple -- --backend dummy
plugin:
	cargo +stable xtask bundle nih_plug_egui_test --release
fmt:
	cargo fmt -p nih_plug_egui_test
