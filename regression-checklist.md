# Regression checklist

Use `simple-egui-plugin` as the standard regression app.

- Build check: `cargo +stable check -p nih_plug_egui_test --example simple`
- Standalone smoke test: `cargo +stable run --release --example simple -- --backend dummy`
- Bundle: `cargo +stable xtask bundle nih_plug_egui_test --release`
- Confirm the editor opens and the gain slider updates host automation.
- Confirm audio passes through with gain applied.
