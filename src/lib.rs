#![warn(clippy::all, rust_2018_idioms)]

mod wrap_app;
pub use wrap_app::CodeChallengeApp;

pub mod apps;
pub mod components;
pub mod helpers;
