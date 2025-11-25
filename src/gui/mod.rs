#![cfg(feature = "ui")]

pub mod app;
pub mod assets;
pub mod components;
pub mod fonts;
pub mod layout;
pub mod theme;

pub use app::run_ui;
