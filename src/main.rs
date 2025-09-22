use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use serde_json;

fn main() -> std::io::Result<()> {
    let file = File::open("game.json")?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    let parsed = serde_json::from_str::<serde_json::Value>(&contents)?;
    for (key, value) in parsed["platform"].as_object().unwrap() {
        println!("{}", key);
        for (key, value) in value.as_object().unwrap() {
            println!("  {}", key.as_str());
            println!("  {}", value["app_id"]);
            println!("  ---");
        }
        println!("---");
    }
    Ok(())
}