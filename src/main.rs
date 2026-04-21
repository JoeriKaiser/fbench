#![allow(non_snake_case)]

mod app;
mod components;
mod config;
mod db;
mod export;
mod filter;
mod hooks;
mod import;
mod llm;
mod services;
mod state;

use app::App;
use dioxus::desktop::muda::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use dioxus::desktop::tao::{dpi::LogicalSize, window::Theme};
use dioxus::desktop::{Config, WindowBuilder};

fn main() {
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            Config::new().with_menu(app_menu()).with_window(
                WindowBuilder::new()
                    .with_title("FBench")
                    .with_theme(Some(Theme::Dark))
                    .with_inner_size(LogicalSize::new(1440.0, 900.0)),
            ),
        )
        .launch(App);
}

fn app_menu() -> Menu {
    let menu = Menu::new();

    let edit_menu = Submenu::new("Edit", true);
    edit_menu
        .append_items(&[
            &PredefinedMenuItem::undo(None),
            &PredefinedMenuItem::redo(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::cut(None),
            &PredefinedMenuItem::copy(None),
            &PredefinedMenuItem::paste(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::select_all(None),
        ])
        .unwrap();

    menu.append_items(&[&edit_menu]).unwrap();

    if cfg!(debug_assertions) {
        let help_menu = Submenu::new("Help", true);

        help_menu
            .append_items(&[&MenuItem::with_id(
                "dioxus-toggle-dev-tools",
                "Toggle Developer Tools",
                true,
                None,
            )])
            .unwrap();

        help_menu
            .append_items(&[&MenuItem::with_id(
                "dioxus-float-top",
                "Float on Top (dev mode only)",
                true,
                None,
            )])
            .unwrap();

        menu.append_items(&[&help_menu]).unwrap();
    }

    menu
}
