#![cfg(target_arch = "wasm32")]

mod app;
mod builder;
mod document;
mod download;
mod editor;
mod form;
mod highlight;
mod options;
mod preview;
mod samples;

pub use app::App;
