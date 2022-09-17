use crate::prelude::*;
use std::env;
use std::fs;
use std::fs::{DirBuilder, File};
use std::io::{BufReader, BufWriter, Read, Write};
use toml::Value;

const DEFAULT_CONFIG: &str = r#"
[master]
directory = "{CUR}/data/master_csv/"
history = "{CUR}/data/history/"
"#;

// toml形式の設定ファイルを読み込む
pub fn load_config() -> Result<Value> {
    // 当プログラムのディレクトリ
    let cur_path_str = env::current_exe().unwrap();
    let cur_path = Path::new(&cur_path_str);
    let cur_dir = cur_path.parent().unwrap().display();

    // 当プログラムのディレクトリ配下に存在する該当拡張子のファイルパスを取得
    let file_paths = glob(&cur_dir.to_string(), "toml", true)?;

    // toml形式変換前の設定文字列
    let mut conf_toml_str = String::new();
    // 全ファイルをテキストで読み込み
    for path in file_paths.iter() {
        conf_toml_str = format!("{}{}", conf_toml_str, get_text(Path::new(&path)));
    }

    // 設定を1件も取得できていなければ
    if conf_toml_str.is_empty() {
        // コンフィグ用ディレクトリを生成
        let path_str = format!("{}//config//config.toml", &cur_dir);
        let path = Path::new(&path_str);
        let dir = path.parent().unwrap();
        DirBuilder::new().recursive(true).create(&dir)?;

        // パスを書き込みモードで開く
        let file = match File::create(&path) {
            Err(e) => panic!("couldn't create {}: {}", path_str, &e.to_string()),
            Ok(file) => file,
        };

        let mut f = BufWriter::new(file);
        conf_toml_str = DEFAULT_CONFIG.to_string();
        match f.write_all(conf_toml_str.as_bytes()) {
            Err(e) => panic!("couldn't write {}: {}", path_str, &e.to_string()),
            Ok(_) => println!("{} writes :{}\n", path_str, conf_toml_str),
        }
    }
    // 文字列内に「{CUR}」が存在すれば、当プログラムが存在するディレクトリとみなして、カレントディレクトリに置換
    conf_toml_str = conf_toml_str.replace("{CUR}", &format!("{}", &cur_dir));

    // 設定をtoml形式に変換して返す
    let value = conf_toml_str.parse::<Value>().unwrap_or_else(|_| {
        panic!(
            "couldn't parse config file to toml format.{}",
            &conf_toml_str
        )
    });
    Ok(value)
}
