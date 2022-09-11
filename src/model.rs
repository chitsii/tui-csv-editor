pub use std::collections::HashMap;

use regex::{Regex, RegexBuilder};
use std::fmt::Display;
pub use tui::widgets::{ListState, TableState};

#[derive(Debug, Clone)]
pub enum DataType {
    Utf8,
    Int64,
    Float64,
    Boolean,
    Raw,
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
            DataType::Raw => {
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
                data_type: DataType::Raw,
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
                if i >= self.values.len() - 1 {
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
                    self.values.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    pub fn edit_row(&mut self) {
        let line = self.state.selected();
        match line {
            Some(idx) => {
                self.values[idx] = vec!["edit".to_owned(); self.schema.columns.len()];
            }
            None => (),
        }
    }

    pub fn infer_field_type(&self, string: &str) -> DataType {
        let boolean_re = RegexBuilder::new(r"^\s*(true)$|^(false)$")
            .case_insensitive(true)
            .build()
            .unwrap();

        let float_re =
            Regex::new(r"^(\s*-?((\d*\.\d+)[eE]?[-\+]?\d*)|[-+]?inf|[-+]?NaN|\d+[eE][-+]\d+)$")
                .unwrap();

        let integer_re = Regex::new(r"^\s*-?(\d+)$").unwrap();

        // 特定順序でregexを適用して合致する型を探す
        if boolean_re.is_match(string) {
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

        // let mut field_dtypes: HashMap<Vec<DataType>, usize> = HashMap::new();
        let mut field_dtypes = HashMap::<String, DataType>::new();

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
                        Some(DataType::Raw) => {
                            field_dtypes.insert(col_name.to_owned(), dtype);
                        }
                        None => {
                            field_dtypes.insert(col_name.to_owned(), dtype);
                        }
                    },
                    DataType::Raw => (),
                }
            }
        }

        for c in self.schema.columns.iter_mut() {
            let d = field_dtypes.get(&c.name).unwrap();
            c.data_type = d.clone();
        }
    }

    pub fn add_row(&mut self) {
        let new_line = vec!["".to_owned(); self.schema.columns.len()];
        self.values.push(new_line);
    }
    pub fn add_column(self) {
        self.schema.push(Column::default());
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

pub struct InputText {
    pub input: String,
    pub current_width: usize,
    pub input_width: HashMap<usize, u16>,
    pub messages: Vec<String>,
    pub lines: usize,
}
