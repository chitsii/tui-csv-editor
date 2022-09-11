use crossterm::{
    event::{
        self,
        DisableMouseCapture,
        EnableMouseCapture,
        Event,
        KeyCode,
        KeyEvent, //, KeyEvent, KeyEventKind, KeyModifiers,
        KeyModifiers,
        KeyboardEnhancementFlags,
        ModifierKeyCode,
        PopKeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::collections::BTreeSet;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text,
    widgets::{Block, Borders, ListItem, Paragraph},
    Terminal,
};
use tui_textarea::{Input, Key, TextArea};

use crate::model::{DataTable, InputText, StatefulList};
use crate::prelude::*;
use crate::ui;

#[derive(Debug, Clone, Copy)]
pub enum ConsoleState {
    Start,
    Select,
    EditTable,
    CheckIntegrity,
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
        //テーブルに表示するデータ準備
        let data = vec![
            vec!["Header1", "Header2", "Header3", "Header4", "Header5"],
            vec!["Row11", "Row12", "1", "2.035", "True"],
            vec!["Row21", "Row22", "2", "3.20", "False"],
            vec!["Row31", "Row32", "3", "111.1", "False"],
        ];
        let mut data_table = DataTable::new(data);

        // イベントループ
        loop {
            self.state = match self.state {
                ConsoleState::Start => ConsoleState::Select,
                ConsoleState::Select => {
                    let items: Vec<ListItem> = vec![ListItem::new("text1"), ListItem::new("text2")];
                    let menu_list = StatefulList::with_items(items.clone());
                    self.selecting(terminal, menu_list).unwrap()
                }
                ConsoleState::EditTable => {
                    // 編集画面のイベントループ
                    self.table_editing(terminal, &mut data_table).unwrap()
                }
                ConsoleState::CheckIntegrity => {
                    println!("Integrity check mode");
                    ConsoleState::EditTable
                }
                ConsoleState::Quit => break,
            };
        }
        Ok(())
    }

    fn table_editing<B: Backend>(
        &self,
        terminal: &mut Terminal<B>,
        data_table: &mut DataTable,
    ) -> io::Result<ConsoleState> {
        let mut selected_indices: BTreeSet<usize> = BTreeSet::new();
        loop {
            terminal.draw(|f| ui::edit(f, data_table))?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => return Ok(ConsoleState::Select),
                    KeyCode::Enter => {
                        self.row_editing(terminal, data_table)?;
                    }
                    KeyCode::Down => data_table.next(),
                    KeyCode::Up => data_table.previous(),
                    KeyCode::Char('r') => data_table.infer_schema(Some(100)),
                    // 行を選択
                    KeyCode::Right => {
                        if let Some(idx) = data_table.state.selected() {
                            selected_indices.insert(idx);
                        }
                    }
                    // 行選択をはずす
                    KeyCode::Left => {
                        if let Some(idx) = data_table.state.selected() {
                            if selected_indices.contains(&idx) {
                                selected_indices.take(&idx);
                            } else {
                                continue;
                            }
                        }
                    }
                    KeyCode::Char('u') => {
                        for idx in &selected_indices {
                            let r = data_table.values[*idx].clone();
                            data_table
                                .values
                                .insert(data_table.state.selected().unwrap(), r)
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    fn row_editing<B: Backend>(
        &self,
        terminal: &mut Terminal<B>,
        data_table: &mut DataTable,
    ) -> io::Result<ConsoleState> {
        // テキストエリアのアクティブ・非アクティブ関数
        fn inactivate(textarea: &mut TextArea<'_>) {
            textarea.set_cursor_line_style(Style::default());
            textarea.set_cursor_style(Style::default());
            let b = textarea
                .block()
                .cloned()
                .unwrap_or_else(|| Block::default().borders(Borders::ALL));
            textarea.set_block(
                b.style(Style::default().fg(Color::DarkGray))
                    .title("非アクティブ"),
            );
        }
        fn activate(textarea: &mut TextArea<'_>) {
            // textarea.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
            textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
            let b = textarea
                .block()
                .cloned()
                .unwrap_or_else(|| Block::default().borders(Borders::ALL));
            textarea.set_block(b.style(Style::default()).title("アクティブ"));
        }

        //表示するカラム名の作成
        let col_names = data_table
            .schema
            .columns
            .iter()
            .map(|c| format!("{}\n [{}]", c.name, c.data_type));
        let mut which: usize = 0;
        let header_len: usize = col_names.len();

        let default_row_data = data_table
            .values
            .get(data_table.state.selected().unwrap())
            .unwrap();

        let mut text_areas: Vec<TextArea> = default_row_data
            .iter()
            .map(|s| TextArea::from([s]))
            .collect();

        // textareaのスタイル初期化
        for t in &mut text_areas {
            inactivate(t);
        }
        activate(&mut text_areas[0]);

        loop {
            terminal.draw(|f| {
                // グローバルの画面領域分割
                let global_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(10), Constraint::Percentage(90)])
                    .margin(2)
                    .split(f.size());

                // ヘルプ情報
                let help_info = Paragraph::new("help information here.")
                    .block(Block::default().borders(Borders::ALL))
                    .alignment(tui::layout::Alignment::Center);

                // エディタ
                let editor_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(10), Constraint::Percentage(90)].as_ref())
                    .margin(2)
                    .split(global_chunks[1]);

                // エディタのヘッダ部分
                let header = col_names.clone().into_iter().map(|name| {
                    Paragraph::new(name)
                        .block(Block::default().borders(Borders::ALL))
                        .alignment(tui::layout::Alignment::Center)
                });
                let constraints = vec![Constraint::Percentage(80 / header_len as u16); header_len];
                let header_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(constraints.as_ref())
                    .split(editor_chunks[0]);

                // エディタの編集部分
                let val_editing_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(80 / 5); 5])
                    .split(editor_chunks[1]);

                // ヘルプ表示
                f.render_widget(help_info, global_chunks[0]);
                // ヘッダ　カラム名\n[型]の表示
                for (paragraph, chunk) in header.zip(header_chunks) {
                    f.render_widget(paragraph, chunk);
                }
                // 編集エリアの表示
                for (textarea, chunk) in text_areas.iter().zip(val_editing_chunks) {
                    let widget = textarea.widget();
                    f.render_widget(widget, chunk);
                }
            })?;

            // キー入力判定
            if let Event::Key(key_event) = event::read()? {
                match key_event {
                    // テーブル編集に戻る
                    KeyEvent {
                        code: KeyCode::Esc, ..
                    } => return Ok(ConsoleState::EditTable),
                    // 保存
                    KeyEvent {
                        code: KeyCode::Char('s'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => {
                        for (idx, cell_value) in data_table.values
                            [data_table.state.selected().unwrap()]
                        .iter_mut()
                        .enumerate()
                        {
                            *cell_value = text_areas[idx].lines().join("\n");
                        }
                        return Ok(ConsoleState::EditTable);
                    }
                    // 編集セルの移動　逆
                    KeyEvent {
                        code: KeyCode::BackTab,
                        modifiers: KeyModifiers::SHIFT,
                        ..
                    } => {
                        inactivate(&mut text_areas[which]);
                        if which == 0 {
                            which = header_len - 1;
                        } else {
                            which -= 1;
                        }

                        activate(&mut text_areas[which]);
                    }
                    // 編集セルの移動　正
                    KeyEvent {
                        code: KeyCode::Tab,
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => {
                        inactivate(&mut text_areas[which]);
                        which = (which + 1) % header_len;
                        activate(&mut text_areas[which]);
                    }
                    // その他の入力は編集エリアに反映
                    key_event => {
                        let input = Input::from(key_event);
                        text_areas[which].input(input);
                    }
                }
            }
        }
    }

    fn selecting<B: Backend>(
        &self,
        terminal: &mut Terminal<B>,
        mut menu_list: StatefulList<ListItem>,
    ) -> io::Result<ConsoleState> {
        loop {
            terminal.draw(|f| ui::select(f, &mut menu_list))?;
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Enter => return Ok(ConsoleState::EditTable),
                    KeyCode::Char('q') => return Ok(ConsoleState::Quit),
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
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES) // https://docs.rs/crossterm/latest/crossterm/event/struct.PushKeyboardEnhancementFlags.html
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
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
