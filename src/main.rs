#![allow(dead_code)]
#![allow(unused_imports)]

mod prelude {
    pub use crate::data_reader::get_string_records;
    pub use crate::utils::{copy, get_text, glob, save_to_file};
    pub use anyhow::Result;
    pub use chrono::{DateTime, Local};
    pub use std::collections::{BTreeMap, BTreeSet};
    pub use std::ffi::OsString;
    pub use std::io;
    pub use std::path::{Path, PathBuf};
    pub use toml::Value;
}

mod config;
mod controller;
mod data_reader;
mod model;
mod ui;
mod utils;
use crate::prelude::*;
use crossterm::{
    event::{
        DisableMouseCapture, EnableMouseCapture, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
pub use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text,
    widgets::{Block, Borders, ListItem, Paragraph},
    Terminal,
};

fn main() -> Result<()> {
    let config = config::load_config()?;

    run_app(config)?;

    Ok(())
}

pub fn run_app(config: Value) -> Result<()> {
    // ターミナルのセットアップ
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES) // https://docs.rs/crossterm/latest/crossterm/event/struct.PushKeyboardEnhancementFlags.html
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = controller::App::new(&config);
    let res = app.run(&mut terminal);

    // ターミナルをrawモードから切り替え
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        PopKeyboardEnhancementFlags
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}
