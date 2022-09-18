#![allow(dead_code)]
#![allow(unused_imports)]

mod prelude {
    pub use crate::data_reader::get_string_records;
    pub use crate::utils::{copy_recursive, get_text, glob, save_to_file};
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

    let mut app = controller::System::new(&config);
    let res = app.run();

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}
