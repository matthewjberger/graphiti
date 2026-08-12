use graphiti::render::{clear_world, populate_world};
use graphiti::{scene_for, schema, theme};
use nightshade::prelude::*;
use protocol::{Command, Event};
use serde_json::Value;

const STARTING_DOCUMENT: &str = include_str!("../../examples/flowchart.json");

#[derive(Default)]
pub struct Board {
    spawned: Vec<Entity>,
}

pub fn initialize(board: &mut Board, world: &mut World) {
    draw(board, world, STARTING_DOCUMENT, "light");
}

pub fn tick(_board: &mut Board, _world: &mut World) {}

pub fn apply_custom(board: &mut Board, world: &mut World, _selected: Option<Entity>, value: Value) {
    let Ok(command) = serde_json::from_value::<Command>(value) else {
        return;
    };
    match command {
        Command::Render { source, theme } => draw(board, world, &source, &theme),
    }
}

fn draw(board: &mut Board, world: &mut World, source: &str, theme_name: &str) {
    let diagram = match schema::parse(source) {
        Ok(diagram) => diagram,
        Err(error) => {
            nightshade_api::offscreen::post_custom(&Event::Failed {
                message: error.to_string(),
            });
            return;
        }
    };
    let selected = theme::theme_by_name(theme_name).unwrap_or_else(theme::theme_light);
    let scene = scene_for(&diagram, &selected);

    let previous = std::mem::take(&mut board.spawned);
    clear_world(world, &previous);
    board.spawned = populate_world(world, &scene, 1.0);

    nightshade_api::offscreen::post_custom(&Event::Rendered {
        width: scene.size.x,
        height: scene.size.y,
        shapes: scene.rects.len() + scene.polygons.len() + scene.circles.len(),
        labels: scene.labels.len(),
    });
}
