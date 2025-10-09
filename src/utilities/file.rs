use std::fs;
use std::str;
use std::io::{self, Read, Seek, SeekFrom};
use std::io::BufReader;
use std::io::prelude::*;
use serde_json;
use ini::Ini;
use std::path::Path;


pub fn read_file(path : &str ) -> Result<String, std::io::Error> {
    let file = fs::File::open(path).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).unwrap();
    Ok(contents)
}

pub fn read_json(path : &str ) -> serde_json::Value {
    let contents = read_file(path).unwrap();
    let parsed = serde_json::from_str::<serde_json::Value>(&contents).ok().unwrap();
    parsed
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

mod custom_iso {
    use super::*;

    const LOGICAL_BLOCK_SIZE: u64 = 2048;
    const PVD_LBA: u64 = 16; // Primary Volume Descriptor'ın konumu

    /// ISO içindeki bir dosya veya dizini temsil eden yapı.
    #[derive(Debug, Clone)]
    pub struct DirectoryEntry {
        pub lba: u64,
        pub size: u32,
        pub is_dir: bool,
        pub identifier: String,
    }

    /// ISO dosya sistemini yöneten ana yapı.
    pub struct IsoFs {
        file: fs::File,
        root_entry: DirectoryEntry,
    }

    impl IsoFs {
        /// Yeni bir IsoFs nesnesi oluşturur. Primary Volume Descriptor'ı okur
        /// ve kök dizini bulur.
        pub fn new(mut file: fs::File) -> io::Result<Self> {
            let mut buffer = vec![0u8; LOGICAL_BLOCK_SIZE as usize];

            // PVD'yi oku
            file.seek(SeekFrom::Start(PVD_LBA * LOGICAL_BLOCK_SIZE))?;
            file.read_exact(&mut buffer)?;

            // PVD'nin doğru olduğunu doğrula (Tip Kodu 1 olmalı)
            if buffer[0] != 1 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Primary Volume Descriptor bulunamadı"));
            }

            // Kök dizin kaydını PVD'den ayrıştır
            let root_record_slice = &buffer[156..190];
            let root_entry = parse_directory_entry(root_record_slice)?;

            Ok(Self { file, root_entry })
        }

        /// Belirtilen yoldaki dosyayı veya dizini bulur.
        pub fn open(&mut self, path: &str) -> io::Result<Option<DirectoryEntry>> {
            let mut current_entry = self.root_entry.clone();
            
            // Yolu bileşenlerine ayır (örn: "/casper/vmlinuz" -> ["casper", "vmlinuz"])
            for component in path.split('/').filter(|s| !s.is_empty()) {
                if !current_entry.is_dir {
                    return Ok(None); // Dosyanın içinde arama yapamayız
                }
                
                // Mevcut dizinin içeriğini oku ve bileşeni ara
                let entries = self.read_dir_contents(&current_entry)?;
                match entries.iter().find(|e| e.identifier.trim_end_matches(';').trim_end_matches('.') == component) {
                    Some(entry) => current_entry = entry.clone(),
                    None => return Ok(None), // Bileşen bulunamadı
                }
            }

            Ok(Some(current_entry))
        }

        /// Bir dizin girdisinin içeriğini okur ve alt girdileri döndürür.
        fn read_dir_contents(&mut self, dir_entry: &DirectoryEntry) -> io::Result<Vec<DirectoryEntry>> {
            let mut buffer = vec![0u8; dir_entry.size as usize];
            self.file.seek(SeekFrom::Start(dir_entry.lba * LOGICAL_BLOCK_SIZE))?;
            self.file.read_exact(&mut buffer)?;

            let mut entries = Vec::new();
            let mut offset = 0;
            while offset < buffer.len() {
                let record_len = buffer[offset] as usize;
                if record_len == 0 {
                    break; // Kayıtların sonu
                }
                let record_slice = &buffer[offset..offset + record_len];
                
                // '.' ve '..' girdilerini atla
                if record_slice[32] > 1 {
                    entries.push(parse_directory_entry(record_slice)?);
                }

                offset += record_len;
            }

            Ok(entries)
        }
        
        /// Ayıklama için bir dosya okuyucusu oluşturur.
        pub fn reader(&mut self, entry: &DirectoryEntry) -> io::Result<impl Read + '_> {
            self.file.seek(SeekFrom::Start(entry.lba * LOGICAL_BLOCK_SIZE))?;
            Ok(self.file.try_clone()?.take(entry.size as u64))
        }
    }

    /// Bir byte diliminden DirectoryEntry'yi ayrıştırır.
    fn parse_directory_entry(slice: &[u8]) -> io::Result<DirectoryEntry> {
        let lba = u32::from_le_bytes(slice[2..6].try_into().unwrap()) as u64;
        let size = u32::from_le_bytes(slice[10..14].try_into().unwrap());
        let flags = slice[25];
        let is_dir = (flags & 2) != 0;
        let identifier_len = slice[32] as usize;
        let identifier = str::from_utf8(&slice[33..33 + identifier_len])
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Geçersiz dosya adı"))?
            .to_string();
        
        // Rock Ridge/Joliet uzantıları için temel bir düzeltme.
        // Genellikle dosya adının sonunda ";1" olur.
        let clean_identifier = identifier.split(';').next().unwrap_or("").to_string();

        Ok(DirectoryEntry { lba, size, is_dir, identifier: clean_identifier })
    }
}

/// Bir ISO dosyasından belirtilen bir dosyayı ayıklar ve hedef yola yazar.
pub fn extract_file(
    iso_path: &str,
    file_path_in_iso: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let iso_file = fs::File::open(iso_path)?;
    let mut fs = custom_iso::IsoFs::new(iso_file)?;

    // ISO içinde belirtilen dosyayı bul
    let file_entry = fs
        .open(file_path_in_iso)?
        .ok_or_else(|| format!("'{}' dosyası ISO içinde bulunamadı.", file_path_in_iso))?;

    if file_entry.is_dir {
        return Err(format!("'{}' bir dosyadır, dizin değil.", file_path_in_iso).into());
    }

    // Hedef dizinin var olduğundan emin ol
    let out_path = Path::new(output_path);
    if let Some(parent_dir) = out_path.parent() {
        fs::create_dir_all(parent_dir)?;
    }
    
    // Dosyayı oku ve çıktı dosyasına yaz
    let mut reader = fs.reader(&file_entry)?;
    let mut output_file = fs::File::create(output_path)?;
    io::copy(&mut reader, &mut output_file)?;

    Ok(())
}