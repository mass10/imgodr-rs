use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::NaiveDateTime;
use walkdir::WalkDir;

/// Rust アプリケーションのエントリーポイント
fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        println!("USAGE:");
        println!("    imgodr-rs <directory(s) or file(s)>");
        std::thread::sleep(std::time::Duration::from_millis(1000));
        return;
    }

    for path in &args {
        find(Path::new(path));
    }

    std::thread::sleep(std::time::Duration::from_millis(1600));
}

/// ファイルまたはディレクトリを再帰的に処理する
fn find(path: &Path) {
    if path.is_dir() {
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                process(entry.path());
            }
        }
    } else if path.is_file() {
        process(path);
    } else {
        println!("パスが存在しません。[{}]", path.display());
    }
}

/// EXIF の DateTimeOriginal を読み取る
fn read_date_taken(path: &Path) -> Option<NaiveDateTime> {
    let file = std::fs::File::open(path).ok()?;
    let mut buf_reader = std::io::BufReader::new(&file);
    let exif_reader = exif::Reader::new();
    let exif = exif_reader.read_from_container(&mut buf_reader).ok()?;

    let field = exif.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY)?;

    let value_str = field.display_value().to_string();

    // EXIF の日時フォーマット: "2014-01-10 23:06:42" or "2014:01:10 23:06:42"
    let normalized = value_str.replace(':', "-");
    // "2014-01-10 23-06-42" -> parse
    let formats = [
        "%Y-%m-%d %H-%M-%S",
        "%Y-%m-%d %H:%M:%S",
        "%Y:%m:%d %H:%M:%S",
    ];

    for fmt in &formats {
        if let Ok(dt) = NaiveDateTime::parse_from_str(&normalized, fmt) {
            return Some(dt);
        }
    }

    // Try original value
    let formats2 = ["%Y-%m-%d %H:%M:%S", "%Y:%m:%d %H:%M:%S"];
    for fmt in &formats2 {
        if let Ok(dt) = NaiveDateTime::parse_from_str(&value_str, fmt) {
            return Some(dt);
        }
    }

    eprintln!(
        "[ERROR] DateTimeOriginal を解析できませんでした。[{}]",
        value_str
    );
    None
}

/// 新しいファイルパスを生成する
fn make_path(original: &Path, date: &NaiveDateTime, index: u32) -> PathBuf {
    let parent = original.parent().unwrap_or(Path::new("."));
    let ext = original
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();

    let date_part = date.format("%Y年%m月%d日 %H時%M分%S秒").to_string();

    if index == 0 {
        parent.join(format!("{}{}", date_part, ext))
    } else {
        parent.join(format!("{} ({}){}", date_part, index, ext))
    }
}

/// ファイルを処理してリネームする
fn process(path: &Path) {
    let Some(date) = read_date_taken(path) else {
        return;
    };

    println!(
        "{} (撮影日時: {})",
        path.display(),
        date.format("%Y-%m-%d %H:%M:%S")
    );

    for i in 0.. {
        let new_path = make_path(path, &date, i);
        if new_path == path {
            // 既に正しい名前
            break;
        }
        if new_path.exists() {
            continue;
        }
        if let Err(e) = fs::rename(path, &new_path) {
            eprintln!(
                "[ERROR] リネーム失敗: {} -> {}: {}",
                path.display(),
                new_path.display(),
                e
            );
        }
        break;
    }
}
