mod animation_controller;

use std::fs::File;
use std::io::BufReader;
use std::path::{Path};

use macroquad::color::LIGHTGRAY;
use macroquad::file::FileError;
use macroquad::input::{is_key_down, is_key_pressed, KeyCode};
use macroquad::math::{Rect, vec2, Vec2};
use macroquad::window::{clear_background, next_frame, screen_height, screen_width};

use tiled::tileset::Tileset;

use macroquad_tiled_redux::{Map, TileSet};
use crate::animation_controller::AnimationRegistry;

enum Direction {
    North,
    East,
    South,
    West,
}

struct GameState {
    pub position: Vec2,
    pub facing: Direction,
    pub zoom: f32,
}

struct Resources {
    pub map: Map,
    // temporary, till animations kick in.
    pub char_tileset: TileSet,
    pub char_animations: AnimationRegistry,
}


// I see three ways to animate things:
// - parts of a Map get animated as a part of Map redraw;
// - Entities with one looping animation just get a `TiAnimationState`.
// - Entities with changing animations (like characters) each get
// an `AnimationController`.

impl GameState {

    pub fn handle_input(&mut self, resources: &Resources) {
        if is_key_pressed(KeyCode::KpAdd) || is_key_down(KeyCode::Key9) {
            self.zoom *= 2.0;
        }
        if (is_key_pressed(KeyCode::Minus) || is_key_down(KeyCode::Key8)) && self.zoom >= 2.0 {
            self.zoom *= 0.5;
        }
        if is_key_down(KeyCode::Key0) || is_key_down(KeyCode::Kp0) {
            self.zoom = 1.0;
            // camera = (map_size.w / 2.0, map_size.h / 2.0);
        }

        // TODO: Check if the terrain is walkable.
        if is_key_pressed(KeyCode::Left) && self.position.x >= 1.0 {
            self.position.x -= 1.0;
            self.facing = Direction::West;
            // camera = (camera.0 - 2.0, camera.1);
        }
        if is_key_pressed(KeyCode::Right) && self.position.x < resources.map.map.width as f32 {
            self.position.x += 1.0;
            self.facing = Direction::East;
            // camera = (camera.0 + 2.0, camera.1);
        }
        if is_key_pressed(KeyCode::Up) && self.position.y >= 1.0 {
            // camera = (camera.0, camera.1 - 2.0);
            self.position.y -= 1.0;
            self.facing = Direction::North;
        }
        if is_key_pressed(KeyCode::Down) && self.position.x < resources.map.map.height as f32 {
            // camera = (camera.0, camera.1 + 2.0);
            self.position.y += 1.0;
            self.facing = Direction::South;
        }
    }

    fn draw(&self, resources: &Resources) {
        clear_background(LIGHTGRAY);

        let screen = Rect::new(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
        );

        let mut source = screen;
        let mut dest = screen;

        source.move_to(vec2(
            self.position.x * resources.map.map.tile_width as f32  - screen_width() / self.zoom / 2.0,
            self.position.y as f32 * resources.map.map.tile_height as f32 - screen_height() / self.zoom / 2.0));

        let source_in_tiles = Rect::new(
            source.x / resources.map.map.tile_width as f32,
            source.y / resources.map.map.tile_height as f32,
            source.w / resources.map.map.tile_width as f32,
            source.h / resources.map.map.tile_height as f32,
        );

        dest.scale(self.zoom, self.zoom);
        for i in 0..resources.map.map.layers.len() {
            resources.map.draw_tiles(i, dest, Some(source_in_tiles));

            if i == 0 {
                let animation = match self.facing {
                    Direction::North => resources.char_animations.get_animation_id("walk-n"),
                    Direction::East => resources.char_animations.get_animation_id("walk-e"),
                    Direction::South => resources.char_animations.get_animation_id("walk-s"),
                    Direction::West => resources.char_animations.get_animation_id("walk-w"),
                };

                if let Some(aid) = animation {
                    self.draw_char(aid, &resources);
                }
            }
        }
    }

    fn draw_char(&self, sprite: u32, resources: &Resources) {

        let dest = Rect::new(
            (screen_width() - resources.map.map.tile_width as f32 * self.zoom) / 2.0,
            (screen_height() - resources.map.map.tile_height as f32 * self.zoom) / 2.0,
            // scale to map's tile size.
            resources.map.map.tile_width as f32 * self.zoom,
            resources.map.map.tile_height as f32 * self.zoom,
        );

        resources.char_tileset.spr(sprite, dest);
    }

}


async fn load_character() -> Result<TileSet, FileError> {
    let path = Path::new("assets/uLPC-drake.tsx");
    let file = File::open(&path).unwrap();
    let reader = BufReader::new(file);

    let tiled_tileset = Tileset::parse_with_path(reader, 1, path).unwrap();
    TileSet::new_async(tiled_tileset)
        .await
}

#[macroquad::main("Texture")]
async fn main() {

    let tilemap = Map::new_async(Path::new("assets/grass/map1.tmx"))
        .await
        .expect("Error loading map");

    let char_tileset = load_character()
        .await
        .expect("Error loading char tileset");
    let char_animations = AnimationRegistry::load(&char_tileset.tileset);

    let mut state = GameState {
        position: vec2(10.0, 10.0),
        facing: Direction::South,
        zoom: 2.0,
    };

    let resources = Resources {
        map: tilemap,
        char_tileset,
        char_animations,
    };

    loop {
        state.draw(&resources);

        state.handle_input(&resources);
        if is_key_down(KeyCode::Q) {
            break;
        }

        next_frame().await
    }
}
