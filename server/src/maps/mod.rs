pub mod grid;
pub mod safe_zone;
pub mod tile;
use crate::items::ItemSpawner;
use std::{collections::HashMap, fs};

fn get_files(dir_path: &str) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    if let Some(file_name) = entry.file_name().to_str() {
                        files.push(file_name.to_string());
                    }
                }
            }
        }
    }
    files
}
pub fn parse_map(path: &str) -> anyhow::Result<grid::Grid> {
    let mut map = grid::Grid::default();
    let file = fs::read_to_string(path)?;
    let lines = file.lines();
    for i in lines {
        let parsed = i.split_whitespace().collect::<Vec<&str>>();
        if parsed[0] == "map" && parsed.len() > 1 {
            map.name = parsed[1].to_string();
        } else if parsed[0] == "tile" && parsed.len() == 6 {
            let min_x = match parsed[1].parse::<isize>() {
                Ok(x) => x,
                Err(e) => {
                    println!("map {} Encountered an error: {}", map.name, e);
                    continue;
                }
            };
            let max_x = match parsed[2].parse::<isize>() {
                Ok(x) => x,
                Err(e) => {
                    println!("map {} Encountered an error: {}", map.name, e);
                    continue;
                }
            };
            let min_y = match parsed[3].parse::<isize>() {
                Ok(x) => x,
                Err(e) => {
                    println!("map {} Encountered an error: {}", map.name, e);
                    continue;
                }
            };
            let max_y = match parsed[4].parse::<isize>() {
                Ok(x) => x,
                Err(e) => {
                    println!("map {} Encountered an error: {}", map.name, e);
                    continue;
                }
            };
            map.add(min_x, max_x, min_y, max_y, parsed[5]);
        } else if parsed[0] == "maxx" && parsed.len() == 2 {
            map.max_x = parsed[1].parse()?;
        } else if parsed[0] == "safe_zone" && parsed.len() >= 5 {
            let min_x = match parsed[1].parse::<isize>() {
                Ok(x) => x,
                Err(e) => {
                    println!("map {} Encountered an error: {}", map.name, e);
                    continue;
                }
            };
            let max_x = match parsed[2].parse::<isize>() {
                Ok(x) => x,
                Err(e) => {
                    println!("map {} Encountered an error: {}", map.name, e);
                    continue;
                }
            };
            let min_y = match parsed[3].parse::<isize>() {
                Ok(x) => x,
                Err(e) => {
                    println!("map {} Encountered an error: {}", map.name, e);
                    continue;
                }
            };
            let max_y = match parsed[4].parse::<isize>() {
                Ok(x) => x,
                Err(e) => {
                    println!("map {} Encountered an error: {}", map.name, e);
                    continue;
                }
            };
            map.add_safe_zone(min_x, max_x, min_y, max_y);
        } else if parsed[0] == "items" && parsed.len() >= 8 {
            let mut names = vec![];
            for i in parsed.iter().skip(7) {
                names.push(i.to_string());
            }
            let item = ItemSpawner::new(
                parsed[1].parse()?,
                parsed[2].parse()?,
                parsed[3].parse()?,
                parsed[4].parse()?,
                parsed[5].parse()?,
                parsed[6].parse()?,
                names,
                map.name.clone(),
            );
            map.item_spawner.push(item);
        }
    }
    Ok(map)
}

pub fn parse_all_maps() -> anyhow::Result<HashMap<String, grid::Grid>> {
    let mut maps = HashMap::new();
    let files = get_files("maps/");
    println!("{:?}", files);
    for i in files {
        if i.ends_with(".map") {
            maps.insert(i.replace(".map", ""), parse_map(&format!("maps/{}", i))?);
        }
    }
    Ok(maps)
}
