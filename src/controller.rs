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
use crate::ui;

#[derive(Debug, Clone)]
pub enum ConsoleState {
    Start,
    Select(Option<String>),
    EditTable(String),
    EditRow(String),
    CheckIntegrity,
    Quit,
}

type DataTables = BTreeMap<OsString, DataTable>;

pub struct App {
    state: ConsoleState,
    data_tables: DataTables,
    archive_dir: String,
    master_dir: String,
}

impl App {
    pub fn new(config: &Value) -> Self {
        let archive_dir = config["master"]["history"].as_str().unwrap();
        let archive_dir = String::from(archive_dir);

        let master_dir = config["master"]["directory"].as_str().unwrap();
        let csv_paths = glob(master_dir, "csv", false).unwrap();

        // data_tablesフィールドの作成
        let mut data_tables: DataTables = BTreeMap::new();
        for path in csv_paths.iter().map(Path::new) {
            let data = get_string_records(path).unwrap();
            let max_len = data.iter().map(|e| e.len()).max().unwrap();
            let mut data: Vec<Vec<String>> = data
                .iter()
                .map(|record| record.iter().map(String::from).collect())
                .collect();
            for record_vec in data.iter_mut() {
                if record_vec.len() < max_len {
                    let diff = max_len - record_vec.len();
                    let mut v = vec![String::new(); diff];
                    record_vec.append(&mut v);
                }
            }
            let data_table = DataTable::new(data);
            let fname = path.file_name().unwrap().to_os_string();
            data_tables.insert(fname, data_table);
        }

        Self {
            state: ConsoleState::Start,
            data_tables,
            master_dir: master_dir.to_string(),
            archive_dir,
        }
    }

    fn get_table(&self, table_name: impl Into<OsString>) -> Option<&DataTable> {
        let key = table_name.into();
        self.data_tables.get(&key)
    }
    fn get_table_mut(&mut self, table_name: impl Into<OsString>) -> Option<&mut DataTable> {
        let key = table_name.into();
        self.data_tables.get_mut(&key)
    }
    fn get_text_for_save(self, table_name: String) -> Option<String> {
        let v = self.get_table(table_name);
        v.map(|data_table| data_table.text())
    }
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        // 画面遷移のイベントループ
        loop {
            self.state = match self.state.clone() {
                ConsoleState::Start => ConsoleState::Select(None),
                ConsoleState::Select(name) => self.select_csv(terminal, name)?,
                ConsoleState::EditTable(name) => self.table_editing(terminal, name)?,
                ConsoleState::EditRow(table_name) => self.row_editing(terminal, table_name)?,
                ConsoleState::CheckIntegrity => {
                    println!("Integrity check mode");
                    ConsoleState::Select(None)
                }
                ConsoleState::Quit => break,
            };
        }
        Ok(())
    }

    fn select_csv<B: Backend>(
        &self,
        terminal: &mut Terminal<B>,
        table_name: Option<String>,
    ) -> Result<ConsoleState> {
        let items: Vec<String> = self
            .data_tables
            .keys()
            .cloned()
            .map(|os_string| os_string.into_string().unwrap())
            .collect();
        let mut menu_list =
            StatefulList::with_items(items.iter().cloned().map(ListItem::new).collect());
        match table_name {
            Some(t) => {
                let idx = items.iter().position(|x| *x == t);
                menu_list.state.select(idx);
            }
            None => menu_list.next(),
        }

        loop {
            terminal.draw(|f| ui::select(f, &mut menu_list))?;
            if let Event::Key(key) = event::read()? {
                match (key.code, key.modifiers) {
                    // 編集
                    (KeyCode::Enter, _) => {
                        let selected = menu_list.state.selected().unwrap();
                        let selected_table_name = &items[selected];
                        return Ok(ConsoleState::EditTable(selected_table_name.to_string()));
                    }
                    // プログラム終了
                    (KeyCode::Char('q'), _) => return Ok(ConsoleState::Quit),
                    // 移動
                    (KeyCode::Down, _) => menu_list.next(),
                    (KeyCode::Up, _) => menu_list.previous(),
                    // 編集したテーブルを保存
                    // TODO: 未保存のテーブルがあるときだけ発動するように
                    (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                        let keys = self.data_tables.keys();
                        let now_str = Local::now().format("%Y-%m-%d-%H%M%S-%Z").to_string();
                        let save_dir = self.archive_dir.clone() + &now_str;

                        for table_name in keys {
                            let save_root_dir = OsString::from(&save_dir);
                            let save_path = Path::new(&save_root_dir).join(Path::new(table_name));
                            save_to_file(self.get_table(table_name).unwrap().text(), save_path)?;
                        }
                        copy_recursive(save_dir, &self.master_dir)?;
                    }
                    _ => {}
                }
            }
        }
    }

    fn table_editing<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        fname: String,
    ) -> Result<ConsoleState> {
        loop {
            let table_name = fname.clone();
            let data_table = self.get_table_mut(table_name.clone()).unwrap();

            terminal.draw(|f| ui::edit(f, data_table))?;

            if let Event::Key(key_event) = event::read()? {
                match key_event {
                    KeyEvent {
                        code: KeyCode::Esc, ..
                    } => return Ok(ConsoleState::Select(Some(table_name))),
                    KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    } => match data_table.state.selected() {
                        Some(_) => return Ok(ConsoleState::EditRow(table_name)),
                        None => {
                            continue;
                        }
                    },
                    KeyEvent {
                        code: KeyCode::Down,
                        ..
                    } => data_table.next(),
                    KeyEvent {
                        code: KeyCode::Up, ..
                    } => data_table.previous(),
                    // スキーマの再推論
                    KeyEvent {
                        code: KeyCode::Char('r'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => data_table.infer_schema(Some(100)),
                    // 行を選択
                    KeyEvent {
                        code: KeyCode::Right,
                        ..
                    } => {
                        if let Some(idx) = data_table.state.selected() {
                            data_table.rows_selected.insert(idx);
                        }
                    }
                    // 行選択をはずす
                    KeyEvent {
                        code: KeyCode::Left,
                        ..
                    } => {
                        if let Some(idx) = data_table.state.selected() {
                            if data_table.rows_selected.contains(&idx) {
                                data_table.rows_selected.take(&idx);
                            }
                        }
                    }
                    // ペースト
                    KeyEvent {
                        code: KeyCode::Char('v'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => {
                        for idx in &data_table.rows_selected {
                            let r = data_table.values[*idx].clone();
                            data_table
                                .values
                                .insert(data_table.state.selected().unwrap() + 1, r)
                        }
                    }
                    //行削除
                    KeyEvent {
                        code: KeyCode::Delete,
                        ..
                    } => {
                        for i in data_table.rows_selected.iter().rev() {
                            data_table.values.remove(*i);
                        }
                        data_table.rows_selected = BTreeSet::new(); //該当行を消したので初期化
                        data_table.state.select(None); // select行が消えた場合はNoneにする
                    }
                    _ => (),
                }
            }
        }
    }

    fn row_editing<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        table_name: String,
    ) -> Result<ConsoleState> {
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
        let data_table = self.get_table_mut(table_name.clone()).unwrap();
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
                    } => return Ok(ConsoleState::EditTable(table_name)),
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
                        return Ok(ConsoleState::EditTable(table_name));
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
}
