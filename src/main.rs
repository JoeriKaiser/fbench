#![allow(non_snake_case)]

mod app;
mod components;
mod config;
mod db;
mod export;
mod hooks;
mod llm;
mod services;
mod state;

use app::App;
use dioxus::desktop::tao::window::Theme;
use dioxus::desktop::{Config, WindowBuilder};

fn main() {
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            Config::new().with_window(
                WindowBuilder::new()
                    .with_title("FBench")
                    .with_theme(Some(Theme::Dark)),
            ),
        )
        .launch(App);
}
