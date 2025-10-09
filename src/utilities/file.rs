use std::fs;
use std::str;
use std::io::{self, Read, Seek, SeekFrom};
use std::io::BufReader;
use std::io::prelude::*;
use pyo3::ffi::c_str;
use serde_json;
use ini::Ini;
use pyo3::prelude::*;
use pyo3::types::PyModule;
use crate::utilities::file;

pub fn read_file(path : &str ) -> Result<String, std::io::Error> {
    let file = fs::File::open(path).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).unwrap();
    Ok(contents)
}

pub fn read_json(path : &str ) -> Option<serde_json::Value> {
    let contents = read_file(path).unwrap();
    let parsed = serde_json::from_str::<serde_json::Value>(&contents).ok()?;
    Some(parsed)
}

// TODO: Need to test
pub fn write_json(path:Option<&str>, data:&serde_json::Value) -> std::io::Result<()> {
    let path = path.unwrap_or("game.json");
    let file = fs::File::create(path)?;
    let mut buf_writer = std::io::BufWriter::new(file);
    let contents = serde_json::to_string_pretty(data).unwrap();
    buf_writer.write_all(contents.as_bytes())?;
    Ok(())
}

pub fn ls(path: &str, search: Option<&str>) -> std::io::Result<Vec<String>> {
    let paths = fs::read_dir(path)?;
    let mut file_list: Vec<String> = Vec::new();
    for path in paths {
        let p = path?.path();
        if let Some(s) = p.to_str() {
            if search.is_some() && !s.contains(search.unwrap()) {
                continue;
            }
            file_list.push(s.to_string());
        }
    }
    Ok(file_list)
}

pub fn get_ini_category(path: &str, category: &str, filter: Option<&str>) -> Vec<String> {
    let ini_path = f!("{}", path);
    let filter = filter.unwrap_or("");
    let config = Ini::load_from_file(ini_path).unwrap();
    let mut game_file_list: Vec<String> = Vec::new();
    if let Some(section) = config.section(Some(category)) {
        for (_, value) in section.iter() {
            if !value.is_empty() && value.ends_with(filter) {
                game_file_list.push(value.to_string());
            }
        }
    }
    game_file_list
}

pub fn get_ini_value(path: &str, category: &str, name: &str) -> String {
    let ini_path = f!("{}", path);
    let config = Ini::load_from_file_noescape(ini_path).unwrap();
    let sec = config.section(Some(category)).unwrap();
    let value = sec.get(name).unwrap();
    value.to_string()
}

/// Basit bir byte dizisi arama fonksiyonu
pub fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

pub fn get_hex(path: &str, offset: u64, end_byte: u8) -> io::Result<String> {
    let mut file = fs::File::open(path)?;

    // 0x837003 konumuna git
    file.seek(SeekFrom::Start(offset))?;

    let mut buffer = Vec::new();
    let mut byte = [0u8; 1];

    // 0x0C değerine kadar oku
    loop {
        let bytes_read = file.read(&mut byte)?;
        if bytes_read == 0 {
            // dosya sonuna geldik
            break;
        }
        if byte[0] == end_byte {
            // 0x0C bulundu, okumayı durdur
            break;
        }
        buffer.push(byte[0]);
    }

    Ok(str::from_utf8(&buffer).unwrap().to_string())
}

// Use iso.py from python for extracting files from ISO
// path: ISO file path (absolute)
// file_name: file to extract from ISO (with path inside ISO)
// output: output path
// Warning: This function not works with 
pub fn write_iso_file(path: &str, file_name:&str, output:&str) -> PyResult<()> {

    let mut pth = std::env::current_dir()?;
    pth.push("src\\utilities\\iso.py");

    Python::attach(|py| {
        let python_iso = PyModule::from_code(
            py,
            c_str!("def extract_file(iso_path, file_path, output_path):
    import pycdlib
    from io import BytesIO
    from pathlib import Path
    iso = pycdlib.PyCdlib()
    iso.open(iso_path)
    extracted = BytesIO()
    iso.get_file_from_iso_fp(extracted, iso_path=file_path)
    Path(output_path.rsplit(\"\\\\\", 1)[0]).mkdir(parents=True, exist_ok=True)
    with open(output_path, 'wb') as f:
        f.write(extracted.getvalue())
    iso.close()"),
            c_str!("iso.py"),
            c_str!("iso"),
    )?.getattr("extract_file")?;
    python_iso.call1((path, file_name, output))?;
    Ok(())
    })
}