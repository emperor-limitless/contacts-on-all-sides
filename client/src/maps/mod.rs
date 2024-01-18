use crate::context::GameContext;
use log::warn;
use std::ops::RangeInclusive;
pub mod ambience;
pub mod grid;
pub mod teleporter;
pub mod tile;

pub fn parse_map(data: &str, ctx: &mut GameContext) -> anyhow::Result<grid::Grid> {
    let mut map = grid::Grid::default();
    let lines = data.lines();

    for i in lines {
        let parsed = i.split_whitespace().collect::<Vec<&str>>();
        if parsed[0] == "tile" && parsed.len() == 6 {
            map.add(
                parsed[1].parse()?,
                parsed[2].parse()?,
                parsed[3].parse()?,
                parsed[4].parse()?,
                parsed[5],
            );
        } else if parsed[0] == "maxx" && parsed.len() == 2 {
            map.max_x = parsed[1].parse()?;
        } else if parsed[0] == "map" && parsed.len() > 1 {
            map.name = parsed[1].to_string();
        } else if parsed[0] == "zone" && parsed.len() >= 6 {
            let mut text = String::new();
            for i in parsed.iter().skip(5) {
                text += &format!("{} ", i);
            }
            map.add_zone(
                parsed[1].parse()?,
                parsed[2].parse()?,
                parsed[3].parse()?,
                parsed[4].parse()?,
                text,
            );
        } else if parsed[0] == "ambience" && parsed.len() >= 6 {
            let path = std::path::Path::new(parsed[5]);
            if !path.exists() {
                continue;
            }
            let mut volume = 0.8;
            if parsed.len() == 7 {
                volume = parsed[6].parse()?;
            }
            map.add_source(
                parsed[1].parse()?,
                parsed[2].parse()?,
                parsed[3].parse()?,
                parsed[4].parse()?,
                parsed[5].to_string(),
                volume,
                ctx,
            )?;
        } else if parsed[0] == "teleporter" && parsed.len() >= 8 {
            let range_x = convert_string_to_range(parsed[5]);
            let range_y = convert_string_to_range(parsed[6]);
            let mut end_x = 0;
            let mut end_y = 0;
            if range_x.is_none() {
                end_x = parsed[5].parse::<isize>()?;
            }
            if range_y.is_none() {
                end_y = parsed[6].parse::<isize>()?;
            }
            map.add_teleporter(
                parsed[1].parse()?,
                parsed[2].parse()?,
                parsed[3].parse()?,
                parsed[4].parse()?,
                end_x,
                end_y,
                parsed[7].to_string(),
                range_x,
                range_y,
            );
        }
    }
    Ok(map)
}

fn convert_string_to_range(value: &str) -> Option<RangeInclusive<isize>> {
    if value.contains("...") {
        let parsed = value.split("...").collect::<Vec<&str>>();
        if parsed.len() != 2 {
            return None;
        }
        let start = match parsed[0].parse::<isize>() {
            Ok(i) => i,
            Err(e) => {
                warn!("Failed to convert range, {}", e);
                return None;
            }
        };
        let end = match parsed[1].parse::<isize>() {
            Ok(i) => i,
            Err(e) => {
                warn!("Failed to convert range, {}", e);
                return None;
            }
        };
        return Some(start..=end);
    }
    None
}
