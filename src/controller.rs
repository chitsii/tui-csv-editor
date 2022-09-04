use crossterm::{
    event::{
        self,
        DisableMouseCapture,
        EnableMouseCapture,
        Event,
        KeyCode, //, KeyEvent, KeyEventKind, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    widgets::ListItem,
    Terminal,
};

use crate::model::{DataTable, StatefulList};
use crate::prelude::*;
use crate::ui;

pub enum ConsoleResult {
    // ToEdit(String),
    ToEdit,
    Quit,
}

#[derive(Debug, Clone, Copy)]
pub enum ConsoleState {
    Start,
    Select,
    Edit,
    IntegrityCheck,
    Quit,
}

pub struct App {
    state: ConsoleState,
}

impl App {
    fn new() -> Self {
        Self {
            state: ConsoleState::Start,
        }
    }
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            self.state = match self.state {
                ConsoleState::Start => {
                    // println!("start mode");
                    ConsoleState::Select
                }
                ConsoleState::Select => {
                    // println!("select mode");
                    let items: Vec<ListItem> = vec![ListItem::new("text1"), ListItem::new("text2")];
                    let menu_list = StatefulList::with_items(items.clone());
                    let res = self.render_select(terminal, menu_list).unwrap();
                    match res {
                        ConsoleResult::ToEdit => ConsoleState::Edit,
                        ConsoleResult::Quit => ConsoleState::Quit,
                    }
                }
                ConsoleState::Edit => {
                    //テーブルに表示するデータ準備
                    let data = vec![
                        vec![
                            "Header1", "Header2", "Header3", "Header4", "Header5", "Header6",
                            "Header7",
                        ],
                        vec!["Row11", "Row12", "Row13", "1", "2.035", "True", "NULL"],
                        vec!["Row21", "Row22", "Row23", "2", "3.20", "False", "NULL"],
                        vec!["Row31", "Row32", "Row33", "3", "111.1", "False", "NULL"],
                    ];
                    let data_table = DataTable::new(data);
                    // 編集画面のイベントループ
                    let res = self.render_edit(terminal, data_table).unwrap();
                    match res {
                        ConsoleResult::Quit => ConsoleState::Select, //ConsoleState::Quit,
                        _ => ConsoleState::Select,
                    }
                }
                ConsoleState::IntegrityCheck => {
                    println!("Integrity check mode");
                    ConsoleState::Edit
                }
                ConsoleState::Quit => break,
            };
        }
        Ok(())
    }

    fn render_edit<B: Backend>(
        &self,
        terminal: &mut Terminal<B>,
        mut data_table: DataTable,
    ) -> io::Result<ConsoleResult> {
        loop {
            terminal.draw(|f| ui::edit(f, &mut data_table))?;
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(ConsoleResult::Quit),
                    KeyCode::Down => data_table.next(),
                    KeyCode::Up => data_table.previous(),
                    _ => {}
                }
            }
        }
    }

    fn render_select<B: Backend>(
        &self,
        terminal: &mut Terminal<B>,
        mut menu_list: StatefulList<ListItem>,
    ) -> io::Result<ConsoleResult> {
        loop {
            terminal.draw(|f| ui::select(f, &mut menu_list))?;
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Enter => return Ok(ConsoleResult::ToEdit),
                    KeyCode::Char('q') => return Ok(ConsoleResult::Quit),
                    KeyCode::Down => menu_list.next(),
                    KeyCode::Up => menu_list.previous(),
                    _ => {}
                }
            }
        }
    }
}

pub fn run_app() -> Result<(), Box<dyn Error>> {
    // ターミナルのセットアップ
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = app.run(&mut terminal);

    // ターミナルをrawモードから切り替え
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}
