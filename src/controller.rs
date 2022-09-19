mod import {
    pub use crossterm::{
        event::{
            self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
            KeyboardEnhancementFlags, ModifierKeyCode, PopKeyboardEnhancementFlags,
            PushKeyboardEnhancementFlags,
        },
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };

    pub use crate::model::{DataTable, StatefulList};
    pub use crate::prelude::*;
    pub use std::fs::{self, DirBuilder, File};
    pub use tui::{
        backend::{Backend, CrosstermBackend},
        layout::{Alignment, Constraint, Direction, Layout, Rect},
        style::{Color, Modifier, Style},
        text,
        widgets::{Block, Borders, ListItem, Paragraph},
        Terminal,
    };
    pub use tui_textarea::{Input, Key, TextArea};
}
use crate::controller::import::*;
use crate::ui::Ui;

#[derive(Debug, Clone)]
pub enum ConsoleState {
    Start,
    Select(Option<String>),
    EditTable(String),
    EditRow(String),
    Quit,
}

pub struct System {
    state: ConsoleState,
    ui: Ui,
}

impl<'a> System {
    pub fn new(config: &'a Value) -> Self {
        let ui = Ui::new(config).unwrap();

        Self {
            state: ConsoleState::Start,
            ui,
        }
    }
    pub fn run(&mut self) -> Result<()> {
        // 画面遷移のイベントループ
        loop {
            self.state = match self.state.clone() {
                ConsoleState::Start => ConsoleState::Select(None),
                ConsoleState::Select(name) => self.select_csv(name)?,
                ConsoleState::EditTable(name) => self.edit_table(name)?,
                ConsoleState::EditRow(name) => self.edit_row(name)?,
                ConsoleState::Quit => break,
            };
        }
        Ok(())
    }

    fn select_csv(&mut self, table_name: Option<String>) -> Result<ConsoleState> {
        self.ui.draw_select_csv(table_name)
    }
    fn edit_table(&mut self, fname: String) -> Result<ConsoleState> {
        self.ui.draw_edit_table(fname.as_str())
    }
    fn edit_row(&mut self, table_name: String) -> Result<ConsoleState> {
        self.ui.draw_edit_row(table_name.as_str())
    }
}
