#![warn(clippy::all, rust_2018_idioms)]

mod wrap_app;
pub use wrap_app::WrapApp;

pub mod apps;
pub mod background_processes;
pub mod code_editor;
pub mod components;
pub mod helpers;
