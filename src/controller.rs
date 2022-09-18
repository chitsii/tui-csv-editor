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
    CheckIntegrity,
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
                ConsoleState::EditTable(name) => self.table_editing(name)?,
                ConsoleState::EditRow(name) => self.row_editing(name)?,
                ConsoleState::CheckIntegrity => {
                    println!("Integrity check mode");
                    ConsoleState::Select(None)
                }
                ConsoleState::Quit => break,
            };
        }
        Ok(())
    }
    /// テーブル編集（レコード単位）
    fn table_editing(&mut self, fname: String) -> Result<ConsoleState> {
        self.ui.draw_table_editing(fname.as_str())
    }

    /// レコード編集（セル単位）
    fn row_editing(&mut self, table_name: String) -> Result<ConsoleState> {
        self.ui.draw_row_editing(table_name.as_str())
    }
    /// テーブル選択
    fn select_csv(&mut self, table_name: Option<String>) -> Result<ConsoleState> {
        self.ui.draw_select(table_name)
    }
}
