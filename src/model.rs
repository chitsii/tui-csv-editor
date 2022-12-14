use crate::prelude::*;

use regex::{Regex, RegexBuilder};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Display;
pub use tui::widgets::{ListState, TableState};

#[derive(Debug, Clone)]
pub enum DataType {
    Utf8,
    Int64,
    Float64,
    Boolean,
    Unknown,
}
impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Boolean => {
                write!(f, "Boolean")
            }
            DataType::Float64 => {
                write!(f, "Float")
            }
            DataType::Int64 => {
                write!(f, "Int")
            }
            DataType::Utf8 => {
                write!(f, "String")
            }
            DataType::Unknown => {
                write!(f, "Unknown")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub data_type: DataType,
}
impl Default for Column {
    fn default() -> Self {
        Self {
            name: String::new(),
            data_type: DataType::Utf8,
        }
    }
}
#[derive(Debug, Clone)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<Column>,
}
impl TableSchema {
    fn push(mut self, new_column: Column) {
        self.columns.push(new_column);
    }
    fn remove(mut self, index: usize) {
        self.columns.remove(index);
    }
}

#[derive(Debug)]
pub struct DataTable {
    pub state: TableState,
    pub rows_selected: BTreeSet<usize>,
    pub schema: TableSchema,
    pub values: Vec<Vec<String>>,
}
impl DataTable {
    pub fn new<S>(data: Vec<Vec<S>>) -> DataTable
    where
        S: Into<String> + Clone + Display,
    {
        // データを行のイテレータに変換
        let mut data_iter = data.into_iter();

        //最初の行はヘッダと想定して取得
        let header = data_iter.next().unwrap();

        // 初期スキーマのcolumnsを作成
        let mut columns = Vec::new();
        for col_name in header {
            columns.push(Column {
                name: col_name.into(),
                data_type: DataType::Unknown,
            });
        }

        let initial_schema = TableSchema {
            name: String::new(),
            columns,
        };

        //値の作成 （data_iterの２行目以降）
        let mut values = Vec::new();
        for v in data_iter {
            let mut row: Vec<String> = Vec::new();
            for vv in v {
                let string_value = vv.into();
                row.push(string_value);
            }
            values.push(row);
        }

        let mut return_value = DataTable {
            state: TableState::default(),
            rows_selected: BTreeSet::new(),
            schema: initial_schema,
            values,
        };

        // 型推論
        return_value.infer_schema(Some(100));
        return_value
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i + 1 >= self.values.len() {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        if self.values.is_empty() {
            self.add_row();
            self.state.select(Some(0));
        } else {
            self.state.select(Some(i));
        }
    }
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.values.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    pub fn infer_field_type(&self, string: &str) -> DataType {
        let empty_re = Regex::new("^$").unwrap();

        let boolean_re = RegexBuilder::new(r"^\s*(true)$|^(false)$")
            .case_insensitive(true)
            .build()
            .unwrap();

        let float_re =
            Regex::new(r"^(\s*-?((\d*\.\d+)[eE]?[-\+]?\d*)|[-+]?inf|[-+]?NaN|\d+[eE][-+]\d+)$")
                .unwrap();

        let integer_re = Regex::new(r"^\s*-?(\d+)$").unwrap();

        // 特定順序でregexを適用して合致する型を探す
        if empty_re.is_match(string) {
            DataType::Unknown
        } else if boolean_re.is_match(string) {
            DataType::Boolean
        } else if float_re.is_match(string) {
            DataType::Float64
        } else if integer_re.is_match(string) {
            DataType::Int64
        } else {
            DataType::Utf8
        }
    }
    pub fn infer_schema(&mut self, max_read_lines: Option<usize>) {
        // 推論に使用するライン数を全行数か設定行数にする
        let len = match max_read_lines {
            Some(v) => std::cmp::min(v, self.values.len()),
            None => self.values.len(),
        };

        let mut field_dtypes = BTreeMap::<String, DataType>::new();

        for row in &self.values[0..len] {
            for (val, col) in row.iter().zip(self.schema.columns.iter()) {
                let dtype = self.infer_field_type(val);
                let col_name = &col.name;
                match dtype {
                    DataType::Utf8 => match field_dtypes.get(col_name) {
                        Some(DataType::Utf8) => (),
                        _ => {
                            field_dtypes.insert(col_name.to_owned(), dtype);
                        }
                    },
                    DataType::Int64 => match field_dtypes.get(col_name) {
                        Some(DataType::Utf8) => (),
                        Some(DataType::Int64) => (),
                        _ => {
                            field_dtypes.insert(col_name.to_owned(), dtype);
                        }
                    },
                    DataType::Float64 => match field_dtypes.get(col_name) {
                        Some(DataType::Utf8) => (),
                        Some(DataType::Int64) => (),
                        Some(DataType::Float64) => (),
                        _ => {
                            field_dtypes.insert(col_name.to_owned(), dtype);
                        }
                    },
                    DataType::Boolean => match field_dtypes.get(col_name) {
                        Some(DataType::Utf8) => (),
                        Some(DataType::Float64) => (),
                        Some(DataType::Int64) => (),
                        Some(DataType::Boolean) => (),
                        Some(DataType::Unknown) => {
                            field_dtypes.insert(col_name.to_owned(), dtype);
                        }
                        None => {
                            field_dtypes.insert(col_name.to_owned(), dtype);
                        }
                    },
                    DataType::Unknown => (),
                }
            }
        }

        for c in self.schema.columns.iter_mut() {
            match field_dtypes.get(&c.name) {
                Some(d) => {
                    c.data_type = d.clone();
                }
                None => c.data_type = DataType::Unknown,
            }
        }
    }
    pub fn add_row(&mut self) {
        let new_line = vec!["".to_owned(); self.schema.columns.len()]; // TODO: スキーマに沿ったデフォルト値生成
        self.values.push(new_line);
    }
    pub fn add_column(self) {
        self.schema.push(Column::default());
    }
    pub fn text(&self) -> String {
        // TODO: ナイーブ
        let text = self
            .values
            .iter()
            .map(|record| record.join(", "))
            .collect::<Vec<String>>()
            .join("\n");

        text
    }
}

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}
impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}
