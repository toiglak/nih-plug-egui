pub mod simple_plugin;
pub mod widgets;

pub use simple_plugin::SimplePlugin;

nih_plug::nih_export_clap!(SimplePlugin);
nih_plug::nih_export_vst3!(SimplePlugin);
