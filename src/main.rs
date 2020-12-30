use std::env;
use std::path;
use regex;
use std::fs;
use chrono;
use zip;
use std::io;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    if args.is_empty() {
        println!("使用方法: seichi915Backup [オプション]");
        println!("  --target | -t");
        println!("    バックアップ対象のディレクトリを指定します。");
        println!("  --destination | -d");
        println!("    バックアップ先のディレクトリを指定します。");
        return;
    }
    let mut backup_target_path: Option<String> = None;
    let mut backup_destination_path: Option<String> = None;
    let str_vec: Vec<&str> = args.iter().map(|s| s as &str).collect::<Vec<&str>>();
    str_vec.iter().for_each(|arg| {
        match arg {
            str if regex::Regex::new(r"--target=\S+").unwrap().is_match(str) => {
                let replaced: String = str.replacen("--target=", "", 1);
                let mut setter = |str| backup_target_path = Some(str);
                setter(replaced);
            },
            str if regex::Regex::new(r"-t=\S+").unwrap().is_match(str) => {
                let replaced: String = str.replacen("-t=", "", 1);
                let mut setter = |str| backup_target_path = Some(str);
                setter(replaced);
            },
            str if regex::Regex::new(r"--destination=\S+").unwrap().is_match(str) => {
                let replaced: String = str.replacen("--destination=", "", 1);
                let mut setter = |str| backup_destination_path = Some(str);
                setter(replaced);
            },
            str if regex::Regex::new(r"-d=\S+").unwrap().is_match(str) => {
                let replaced: String = str.replacen("-d=", "", 1);
                let mut setter = |str| backup_destination_path = Some(str);
                setter(replaced);
            },
            _ => {
                println!("不明なオプションです: {}", arg);
                return;
            }
        }
    });
    if backup_target_path.is_none() {
        println!("バックアップ対象のディレクトリを指定してください。");
        return;
    }
    if backup_destination_path.is_none() {
        println!("バックアップ先のディレクトリを指定してください。");
        return;
    }
    let unwrapped_backup_target_path = backup_target_path.unwrap();
    let unwrapped_backup_destination_path = backup_destination_path.unwrap();
    if !path::Path::new(&unwrapped_backup_target_path).exists() || !path::Path::new(&unwrapped_backup_target_path).is_dir() {
        println!("バックアップ対象 {} は存在しないか、ディレクトリではありません。", unwrapped_backup_target_path);
    }
    if !path::Path::new(&unwrapped_backup_destination_path).exists() || !path::Path::new(&unwrapped_backup_destination_path).is_dir() {
        match fs::create_dir_all(&unwrapped_backup_destination_path) {
            Err(why) => panic!("バックアップ先のディレクトリを作成できませんでした: {}", why),
            Ok(_) => {}
        }
    }
    let mut sorted_file_names: Vec<String> = fs::read_dir(&unwrapped_backup_destination_path).unwrap().filter_map(|entry| {
       let entry = entry.ok().unwrap();
        if entry.file_type().ok().unwrap().is_file() {
            Some(entry.file_name().into_string().unwrap())
        } else {
            None
        }
    }).collect();
    sorted_file_names.sort();
    while fs::read_dir(&unwrapped_backup_destination_path).unwrap().count() > 9 {
        match fs::remove_file(format!("{}/{}", &unwrapped_backup_destination_path, sorted_file_names.remove(0))) {
            Err(why) => panic!("古いバックアップファイルの削除に失敗しました: {}", why),
            Ok(_) => {}
        }
    }
    let filename = chrono::offset::Local::now().format("%Y-%m-%d-%H-%M-%S.zip").to_string();
    let zip_filepath = format!("{}/{}", unwrapped_backup_destination_path.as_str(), filename);
    let file = fs::File::create(zip_filepath).unwrap();
    let mut zip_writer = zip::ZipWriter::new(file);
    add_files(&mut zip_writer, unwrapped_backup_target_path);
    zip_writer.finish().unwrap();
}

fn add_files(zip_writer: &mut zip::ZipWriter<fs::File>, directory: String) {
    for dir_entry in fs::read_dir(directory).unwrap() {
        let path = dir_entry.unwrap().path();
        if path.is_dir() {
            add_directory(zip_writer, path.file_name().unwrap().to_os_string().into_string().unwrap(), path.into_os_string().into_string().unwrap());
        } else {
            let filename = path.file_name().unwrap().to_os_string().into_string().unwrap();
            let filepath = path.into_os_string().into_string().unwrap();
            println!("バックアップ中: {}", &filepath);
            let mut src = fs::File::open(filepath).unwrap();
            zip_writer.start_file(filename, zip::write::FileOptions::default()).unwrap();
            io::copy(&mut src, zip_writer).unwrap();
        }
    }
}

fn add_directory(zip_writer: &mut zip::ZipWriter<fs::File>, dir_name: String, directory: String) {
    for dir_entry in fs::read_dir(directory).unwrap() {
        let path = dir_entry.unwrap().path();
        if path.is_dir() {
            add_directory(zip_writer, format!("{}/{}", dir_name, path.file_name().unwrap().to_os_string().into_string().unwrap()), path.into_os_string().into_string().unwrap());
        } else {
            let filename = path.file_name().unwrap().to_os_string().into_string().unwrap();
            let filepath = path.into_os_string().into_string().unwrap();
            println!("バックアップ中: {}", &filepath);
            let mut src = fs::File::open(filepath).unwrap();
            zip_writer.start_file(format!("{}/{}", dir_name, filename), zip::write::FileOptions::default()).unwrap();
            io::copy(&mut src, zip_writer).unwrap();
        }
    }
}
