
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{self, Path, PathBuf};
use crate::utilities;
use crate::utilities::file::get_hex;

pub fn get_recent_games(path: &str) -> Vec<String> {
    let pth =  format!("{}\\SYSTEM\\ppsspp.ini", path);
    return utilities::file::get_ini_category(&pth, "Recent", Some(".iso"));
}

pub fn get_shared_games(path: &str) -> Vec<String> {
    let pth =  format!("{}\\SYSTEM\\ppsspp.ini", path);
    let shared = utilities::file::get_ini_value(&pth, "General", "RemoteISOSharedDir").replace("\\", "\\"); 
    let game_list = utilities::file::ls(&shared, Some(".iso")).unwrap();
    return game_list;
}

pub struct PPSSPPGame {
    pub id: String,
    pub title: String,
    pub version: String,
    pub path: String,
    pub icon: PathBuf,
    pub thumbnail: PathBuf,
    pub params: PathBuf,
}

pub fn get_game(path: &str) -> Result<PPSSPPGame, std::io::Error> {
    let mut game = PPSSPPGame { 
        id: String::from(""),
        title: String::from(""),
        version: String::from(""),
        path: path.to_string(),
        icon: PathBuf::from("temp\\PIC1.PNG"),
        thumbnail: PathBuf::from("temp\\PIC1.PNG"),
        params: PathBuf::from("temp\\PARAM.SFO"),
    };

    game.id = get_hex(path, 0x8373, 0x7C)?;
    game.icon = PathBuf::from(format!("temp\\psp\\{}\\icon.png", game.id));
    game.thumbnail = PathBuf::from(format!("temp\\psp\\{}\\thumb.png", game.id));
    game.params = PathBuf::from(format!("temp\\psp\\{}\\params.sfo", game.id));

    // get game icon
    utilities::file::write_iso_file(path, "/PSP_GAME/ICON0.PNG", game.icon.to_str().unwrap())?;
    // get game thumbnail
    utilities::file::write_iso_file(path, "/PSP_GAME/PIC1.PNG", game.thumbnail.to_str().unwrap())?;
    // get game info (i.e. title, id, version)
    utilities::file::write_iso_file(path, "/PSP_GAME/PARAM.SFO", game.params.to_str().unwrap())?;

    // read game info from PARAM.SFO
    game.title = get_param_data(game.params.to_str().unwrap())?;

    Ok(game)
}

// TODO: get version too
fn get_param_data(path: &str) -> Result<String, std::io::Error> {
    let mut file = File::open(path)?;

    // Dosyayı tamamen oku
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    // Aranacak dizi: "TITLE" (hex: 54 49 54 4C 45)
    let pattern: [u8; 5] = [0x54, 0x49, 0x54, 0x4C, 0x45];

    let mut result_text = String::new();

    if let Some(pos) = utilities::file::find_bytes(&data, &pattern) {

        // TITLE'dan itibaren oku
        let mut extracted = Vec::new();
        let mut zero_count = 0;

        for &byte in &data[pos..] {
            extracted.push(byte);
            if byte == 0x00 {
                zero_count += 1;
                if zero_count >= 16 {
                    break;
                }
            } else {
                zero_count = 0;
            }
        }

        // "TITLE" ve 16 sıfırı at
        let mut content = extracted[pattern.len()..].to_vec();

        // sondaki 16 sıfırı sil
        while content.ends_with(&[0x00]) {
            content.pop();
        }

        // son 0x80 değerini bul
        if let Some(last_80_pos) = content.iter().rposition(|&b| b == 0x80) {
            let after_80 = &content[last_80_pos + 1..];

            // 0x00 (boşluk) byte'larını kaldır
            let cleaned: Vec<u8> = after_80.iter().cloned().filter(|&b| b != 0x00).collect();

            // Eğer okunabilir bir metinse:
            if let Ok(text) = String::from_utf8(cleaned.clone()) {
                result_text = text;
            }

        } else {
            println!("Content içinde hiç 0x80 bulunamadı!");
        }

    } else {
        println!("'TITLE' bulunamadı.");
    }

    Ok(result_text)
}