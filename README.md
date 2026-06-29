# nih-plug-egui test

This repository contains a minimal `nih-plug` audio effect using the upstream
`nih_plug_egui` editor helper.

The previous adapter experiment and vendored workspace have been removed. The
crate now builds a simple gain plugin with an egui editor backed by `EguiState`
and `widgets::ParamSlider`.

## Build

```sh
cargo +stable check -p nih_plug_egui_test --example simple
cargo +stable xtask bundle nih_plug_egui_test --release
```

The bundled plugin is written to `target/bundled/`.
