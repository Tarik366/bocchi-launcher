use directories::UserDirs;

use crate::utilities::file::{self, read_json};
mod utilities;
mod ppsspp;
mod ui;

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


fn get_ppssspp_games() -> std::io::Result<()> {
    
    let recent_psp_game_paths = &ppsspp::get_recent_games("C:\\Users\\Tarık\\Documents\\PPSSPP\\PSP"); // dosya yolunu buraya yaz
    let mut psp_game_paths = recent_psp_game_paths.clone(); // recent ve shared oyunları birleştir
    for game in &ppsspp::get_shared_games("C:\\Users\\Tarık\\Documents\\PPSSPP\\PSP") {
        if !psp_game_paths.contains(game) {
            psp_game_paths.push(game.clone());
        }
    }
    for game_path in &psp_game_paths {
        let game = ppsspp::get_game(game_path)?;
        println!("Game ID: {}, Title: {}, Version: {}, Path: {}, Icon: {:?}, Thumbnail: {:?}, Params: {:?}", 
            game.id, game.title, game.version, game.path, game.icon, game.thumbnail, game.params);
    }

    read_json("game.json");

    // TODO: print games to json file

    Ok(())
}

fn main() -> iced::Result {
    get_ppssspp_games().unwrap();
    
    Ok(())
}

