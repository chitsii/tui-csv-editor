use crate::prelude::*;
use std::ffi::OsString;
use std::fs::{self, DirBuilder, File};
use std::io::{BufReader, Read, Write};

/// 対象ディレクトリを再帰的に探索して、指定拡張子のファイルのパスの配列を返す
pub fn glob(target: &str, target_ext: &str, recursive: bool) -> Result<Vec<OsString>> {
    let mut files: Vec<OsString> = Vec::new();
    for file_path in fs::read_dir(target)? {
        let file_path = file_path?.path();
        if file_path.is_dir() {
            if recursive {
                let mut _files = glob(&file_path.display().to_string(), target_ext, true)?;
                files.append(&mut _files);
            } else {
                continue;
            }
        } else if let Some(file_ext) = file_path.extension() {
            if file_ext == target_ext {
                files.push(file_path.into_os_string());
            }
        }
    }
    Ok(files)
}

/// 指定パスのファイルをStringに読み出して返す
pub fn get_text(path: &Path) -> String {
    let display = path.display();
    // 読み込み専用モード
    let f = match File::open(&path) {
        Err(e) => panic!("couldn't open {}: {}", display, &e.to_string()),
        Ok(f) => f,
    };
    // バッファリングされたストリーム
    let mut br = BufReader::new(f);
    let mut text = String::new();
    if let Err(e) = br.read_to_string(&mut text) {
        panic!("couldn't read {}: {}", display, &e.to_string())
    }
    text
}

pub fn save_to_file(content: String, path: PathBuf) -> Result<()> {
    let dir = path.parent().unwrap();
    // 指定ディレクトリが存在しない場合、作る
    if !&dir.exists() {
        DirBuilder::new().recursive(true).create(&dir)?;
    }
    //write-onlyモードでファイルに書き込み
    let mut file = File::create(path)?;
    write!(file, "{}", content)?;
    file.flush()?;
    Ok(())
}

pub fn copy_recursive<U: AsRef<Path>, V: AsRef<Path>>(
    from: U,
    to: V,
) -> Result<(), std::io::Error> {
    let mut stack = Vec::new();
    stack.push(PathBuf::from(from.as_ref()));

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();

    while let Some(working_path) = stack.pop() {
        // println!("process: {:?}", &working_path);

        // 相対パス生成
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // destinationがなければ作成
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            // println!(" mkdir: {:?}", dest);
            fs::create_dir_all(&dest)?;
        }

        for entry in fs::read_dir(working_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                match path.file_name() {
                    Some(filename) => {
                        let dest_path = dest.join(filename);
                        // println!("  copy: {:?} -> {:?}", &path, &dest_path);
                        fs::copy(&path, &dest_path)?;
                    }
                    None => {
                        println!("failed: {:?}", path);
                    }
                }
            }
        }
    }

    Ok(())
}
