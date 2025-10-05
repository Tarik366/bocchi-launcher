use directories::UserDirs;

use crate::utilities::file::get_hex;

mod utilities;
mod ppsspp;

#[macro_use]
extern crate fstrings;

struct Paths {
    ppsspp: String,
    dolphin: String,
    steam: String,
    epic: String,
    gog: String,
}

fn get_games(path:Option<&str>) -> std::io::Result<()> {
    let path = path.unwrap_or("/");
    let balık: Paths;
    if let Some(user_dirs) = UserDirs::new() {
        balık = Paths {
            ppsspp: f!("{}\\PPSSPP\\PSP", user_dirs.document_dir().unwrap().to_str().unwrap()),
            dolphin: f!("C:\\Users\\Tarık\\Documents\\Dolphin Emulator\\Games"),
            steam: f!("C:\\Program Files (x86)\\Steam\\steamapps\\common"),
            epic: f!("C:\\Program Files\\Epic Games"),
            gog: f!("C:\\Program Files (x86)\\GOG Galaxy\\Games"),
        };

        println!("PPSSPP Games: {}", balık.ppsspp);
    }
    // PPSSPP games
    // C:\Users\Tarık\Documents\PPSSPP\PSP\GAME
    
    Ok(())
}

fn main() -> std::io::Result<()> {
    let psp_game_paths = &ppsspp::get_shared_games("C:\\Users\\Tarık\\Documents\\PPSSPP\\PSP"); // dosya yolunu buraya yaz
    for game_path in psp_game_paths {
        let game = ppsspp::get_game(game_path)?;
        println!("Game ID: {}, Title: {}, Version: {}, Path: {}, Icon: {:?}, Thumbnail: {:?}, Params: {:?}", 
            game.id, game.title, game.version, game.path, game.icon, game.thumbnail, game.params);
    }
    Ok(())
}

