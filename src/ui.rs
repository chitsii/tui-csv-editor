use crate::model::{DataTable, StatefulList};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table,
        Wrap,
    },
    Frame,
};
use tui_textarea::TextArea;

pub fn editor_title<'a>() -> Paragraph<'a> {
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

pub fn edit<B: Backend>(f: &mut Frame<B>, data_table: &mut DataTable) {
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
    let value_headers = data_table.schema.columns.iter().map(|c| {
        Cell::from(format!("{}\n [{}]", c.name, c.data_type))
            .style(Style::default().fg(Color::Gray))
    });
    let idx_header = [Cell::from("")].into_iter();
    let header_cells = idx_header.chain(value_headers);
    let header = Row::new(header_cells).style(header_style).height(2);

    //表示するデータの作成
    let rows = data_table.values.iter().enumerate().map(|(index, item)| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;

        //9,999,999までindex可能
        let mut index_str = format!("{:>7}", index);

        match data_table.rows_selected.contains(&index) {
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
    let mut value_widths = vec![Constraint::Length(30); data_table.schema.columns.len()];
    widths.append(&mut value_widths);

    let t = Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::LightMagenta))
                .title(data_table.schema.name.clone()),
        )
        .highlight_style(selected_style)
        .widths(&widths);
    //テーブル作成完了

    // helpを作成
    let title = editor_title();

    // 表示
    f.render_widget(title, rects[0]);
    f.render_stateful_widget(t, rects[1], &mut data_table.state);
}

pub fn select<B: Backend>(f: &mut Frame<B>, menu_list: &mut StatefulList<ListItem>) {
    // 画面領域の分割
    let rects = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(50)].as_ref())
        .margin(5)
        .split(f.size());

    let items = menu_list.items.clone();
    let items_widget = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("👉  ");
    // 表示
    f.render_stateful_widget(items_widget, rects[0], &mut menu_list.state);
}
