use crate::controller::ConsoleState;
use crate::model::{DataTable, DataTables, StatefulList};
use crate::prelude::*;
pub use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
        KeyboardEnhancementFlags, ModifierKeyCode, PopKeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Stdout};
use toml::Value;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, List, ListItem, Paragraph, Row, Table},
    Frame, Terminal,
};
use tui_textarea::{Input, TextArea};

pub struct Ui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    pub model: DataTables,
}

impl Ui {
    pub fn new(config: &Value) -> Result<Self> {
        // TUI用のターミナル用意
        enable_raw_mode()?;

        let mut stdout = io::stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            EnableMouseCapture,
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES) // https://docs.rs/crossterm/latest/crossterm/event/struct.PushKeyboardEnhancementFlags.html
        )
        .unwrap();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).unwrap();

        let model = DataTables::new(config);

        Ok(Ui { terminal, model })
    }
}

// 行編集画面
impl Ui {
    pub fn draw_edit_row(&mut self, table_name: &str) -> Result<ConsoleState> {
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
        let table = self.model.get_table_mut(table_name).unwrap();
        let mut which: usize = 0;
        let default_row_data = table.values.get(table.state.selected().unwrap()).unwrap();

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
            self.terminal.draw(|f| {
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

                let header = table.header().into_iter().map(|name| {
                    Paragraph::new(name)
                        .block(Block::default().borders(Borders::ALL))
                        .alignment(tui::layout::Alignment::Center)
                });
                let constraints = vec![
                    Constraint::Percentage(80 / table.header().len() as u16);
                    table.header().len()
                ];
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
                    } => return Ok(ConsoleState::EditTable(table_name.to_string())),
                    // 保存
                    KeyEvent {
                        code: KeyCode::Char('s'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => {
                        for (idx, cell_value) in table.values[table.state.selected().unwrap()]
                            .iter_mut()
                            .enumerate()
                        {
                            *cell_value = text_areas[idx].lines().join("\n");
                        }
                        return Ok(ConsoleState::EditTable(table_name.to_string()));
                    }
                    // 編集セルの移動　逆
                    KeyEvent {
                        code: KeyCode::BackTab,
                        modifiers: KeyModifiers::SHIFT,
                        ..
                    } => {
                        inactivate(&mut text_areas[which]);
                        if which == 0 {
                            which = table.header().len() - 1;
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
                        which = (which + 1) % table.header().len();
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

// テーブル編集画面
impl Ui {
    pub fn draw_edit_table(&mut self, table_name: &str) -> Result<ConsoleState> {
        let table = self.model.get_table_mut(table_name).unwrap();

        loop {
            self.terminal.draw(|f| Ui::table_editing(f, table))?;

            if let Event::Key(key_event) = event::read()? {
                match key_event {
                    KeyEvent {
                        code: KeyCode::Esc, ..
                    } => return Ok(ConsoleState::Select(Some(table_name.to_string()))),
                    KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    } => match table.state.selected() {
                        Some(_) => return Ok(ConsoleState::EditRow(table_name.to_string())),
                        None => {
                            continue;
                        }
                    },
                    KeyEvent {
                        code: KeyCode::Down,
                        ..
                    } => table.next(),
                    KeyEvent {
                        code: KeyCode::Up, ..
                    } => table.previous(),
                    // スキーマの再推論
                    KeyEvent {
                        code: KeyCode::Char('r'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => table.infer_schema(Some(100)),
                    // 行を選択
                    KeyEvent {
                        code: KeyCode::Right,
                        ..
                    } => {
                        if let Some(idx) = table.state.selected() {
                            table.rows_selected.insert(idx);
                        }
                    }
                    // 行選択をはずす
                    KeyEvent {
                        code: KeyCode::Left,
                        ..
                    } => {
                        if let Some(idx) = table.state.selected() {
                            if table.rows_selected.contains(&idx) {
                                table.rows_selected.take(&idx);
                            }
                        }
                    }
                    // ペースト
                    KeyEvent {
                        code: KeyCode::Char('v'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => {
                        for idx in &table.rows_selected {
                            let r = table.values[*idx].clone();
                            table.values.insert(table.state.selected().unwrap() + 1, r)
                        }
                    }
                    //行削除
                    KeyEvent {
                        code: KeyCode::Delete,
                        ..
                    } => {
                        for i in table.rows_selected.iter().rev() {
                            table.values.remove(*i);
                        }
                        table.rows_selected = BTreeSet::new(); //該当行を消したので初期化
                        table.state.select(None); // select行が消えた場合はNoneにする
                    }
                    _ => (),
                }
            }
        }
    }
    fn table_editing(f: &mut Frame<CrosstermBackend<Stdout>>, table: &mut DataTable) {
        // 画面領域の分割
        let rects = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(10), Constraint::Percentage(90)].as_ref())
            .margin(5)
            .split(f.size());

        // テーブル作成開始
        //行を選択した時のスタイル
        let selected_style = Style::default().add_modifier(Modifier::REVERSED);

        //表示するヘッダのの作成
        let header_style = Style::default()
            .bg(Color::Black)
            .add_modifier(Modifier::BOLD);

        let header = table.schema.columns.iter().map(|c| {
            Cell::from(format!("{}\n [{}]", c.name, c.data_type))
                .style(Style::default().fg(Color::Gray))
        });

        let idx_column = [Cell::from("")].into_iter();
        let header_cells = idx_column.chain(header);
        let header = Row::new(header_cells).style(header_style).height(2);

        //表示するデータの作成
        let rows = table.values.iter().enumerate().map(|(index, item)| {
            let height = item
                .iter()
                .map(|content| content.chars().filter(|c| *c == '\n').count())
                .max()
                .unwrap_or(0)
                + 1;

            //9,999,999までindex可能
            let mut index_str = format!("{:>7}", index);

            match table.rows_selected.contains(&index) {
                true => {
                    index_str += "🎈";
                }
                false => {}
            }

            let idx_cell =
                [Cell::from(index_str).style(Style::default().fg(Color::DarkGray))].into_iter();
            let value_cells = item.iter().map(|c| Cell::from(c.clone()));
            let cells = idx_cell.chain(value_cells);
            Row::new(cells).height(height as u16).bottom_margin(0)
        });

        // 表示するカラムのwidthsを動的に作る
        // 1カラム目は7(index), 残りはvalueで一律長さ30
        let mut widths = vec![Constraint::Length(10)];
        let mut value_widths = vec![Constraint::Length(30); table.schema.columns.len()];
        widths.append(&mut value_widths);

        let t = Table::new(rows)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::LightMagenta))
                    .title(table.schema.name.clone()),
            )
            .highlight_style(selected_style)
            .widths(&widths);

        // helpを作成
        let title = Ui::editor_title();

        // 表示
        f.render_widget(title, rects[0]);
        f.render_stateful_widget(t, rects[1], &mut table.state);
    }
    fn editor_title<'a>() -> Paragraph<'a> {
        let text = vec![
            Spans::from(vec![
                Span::raw("CSV Editor"),
                Span::styled("操作方法", Style::default().fg(Color::LightCyan)),
            ]),
            Spans::from(Span::styled("Second line", Style::default().fg(Color::Red))),
        ];
        Paragraph::new(text).alignment(Alignment::Center).block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        )
    }
}

// CSV選択画面
impl Ui {
    pub fn draw_select_csv(&mut self, table_name: Option<String>) -> Result<ConsoleState> {
        let items: Vec<&str> = self.model.tables();
        let mut list = StatefulList::with_items(items.iter().cloned().map(ListItem::new).collect());

        match table_name {
            Some(t) => {
                let idx = items.iter().position(|x| *x == t);
                list.state.select(idx);
            }
            None => list.next(),
        }

        loop {
            self.terminal.draw(|f| {
                Ui::select_csv_widgets(f, &mut list);
            })?;

            if let Event::Key(key) = event::read()? {
                match (key.code, key.modifiers) {
                    // 編集
                    (KeyCode::Enter, _) => {
                        let selected = list.state.selected().unwrap();
                        let selected_table_name = &items[selected];
                        return Ok(ConsoleState::EditTable(selected_table_name.to_string()));
                    }
                    // プログラム終了
                    (KeyCode::Char('q'), _) => return Ok(ConsoleState::Quit),
                    // 移動
                    (KeyCode::Down, _) => list.next(),
                    (KeyCode::Up, _) => list.previous(),
                    // 編集したテーブルを保存
                    (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                        self.model.save()?;
                    }
                    _ => {}
                }
            }
        }
    }
    fn select_csv_widgets(
        f: &mut Frame<CrosstermBackend<Stdout>>,
        list: &mut StatefulList<ListItem>,
    ) {
        // 画面領域の分割
        let rects = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(50)].as_ref())
            .margin(5)
            .split(f.size());
        let items = list.items.clone();
        let widgets = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("List"))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("👉  ");
        f.render_stateful_widget(widgets, rects[0], &mut list.state);
    }
}

// デストラクタ
impl Drop for Ui {
    fn drop(&mut self) {
        // ターミナルを元に戻す
        disable_raw_mode().unwrap();
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            PopKeyboardEnhancementFlags
        )
        .unwrap();
        self.terminal.show_cursor().unwrap();
    }
}
