use amethyst::{
    assets::Handle,
    core::Transform,
    prelude::*,
    renderer::{SpriteRender, SpriteSheet},
};
use either::Either;
use log::debug;
use nalgebra::{UnitQuaternion, Vector3};
use tmx::{
    map::{self, Map, TilesetKind},
    tileset::Tileset,
};

use crate::{systems::CameraTarget, world};
use world::MapsConfig;
use std::path::Path;

const AIR_TILE_TYPE: &str = "air";
const PLATFORM_TILE_TYPE: &str = "platform";
const PLAYER_TILE_TYPE: &str = "player";

pub fn initialize_tiles(world: &mut World, sprite_sheet: Handle<SpriteSheet>) {
    let filepath = world.get_mut::<MapsConfig>().unwrap().default.canonicalize().unwrap();
    debug!("loading .tmx file from {}", filepath.display());
    let tmx = Map::from_json(&std::fs::read_to_string(&filepath).unwrap())
        .unwrap_or_else(|e| panic!("{0} ({0:?})", e));
    initialize_tiles_with(world, sprite_sheet, &filepath, tmx);
}

pub fn initialize_tiles_with(world: &mut World, sprite_sheet: Handle<SpriteSheet>, tiles_filepath: &Path, tiles: Map) {
    assert_eq!(tiles.orientation, map::Orientation::Orthogonal);
    assert_eq!(tiles.render_order, map::RenderOrder::RightDown);
    assert_eq!(tiles.tilesets.len(), 1);
    let owned_tileset;
    let tileset = match &tiles.tilesets[0].kind {
        TilesetKind::Embedded(tileset) => tileset,
        TilesetKind::External { source } => {
            let tileset_src = tiles_filepath.parent().unwrap().join(source);
            debug!("loading external .tsx file from {}", tileset_src.display());
            owned_tileset = Tileset::from_xml(&std::fs::read_to_string(tileset_src).unwrap())
                .unwrap_or_else(|e| panic!("{0} ({0:?})", e));
            &owned_tileset
        }
    };

    let tile_iter = tiles.layers.iter().flat_map(|layer| {
        use tmx::layer::LayerData::*;
        match &layer.data {
            Tiles(tiles) => {
                let tmx::layer::Layer {
                    x: layer_x,
                    y: layer_y,
                    width: layer_width,
                    ..
                } = *layer;
                Either::Left(tiles.iter().enumerate().map(move |(i, v)| {
                    let i = i as i32;
                    (
                        (layer_x + (i % layer_width), layer_y + (i / layer_width)),
                        v,
                    )
                }))
            }
            Chunks(chunks) => Either::Right(chunks.iter().flat_map(|chunk| {
                let tmx::layer::Chunk {
                    x: chunk_x,
                    y: chunk_y,
                    width: layer_width,
                    ..
                } = *chunk;
                chunk.data.iter().enumerate().map(move |(i, v)| {
                    let i = i as i32;
                    (
                        (
                            chunk_x as i32 + (i % (layer_width as i32)),
                            chunk_y as i32 + (i / (layer_width as i32)),
                        ),
                        v,
                    )
                })
            })),
        }
    });

    tile_iter.for_each(|((x, y), tile)| {
        if tile.gid() == 0 {
            return;
        }
        // we simply assume the sprite sheet is our own.
        let tile = &tileset.tiles[(tile.gid() - tiles.tilesets[0].first_gid) as usize];
        let sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet.clone(),
            sprite_number: tile.id as usize,
        };
        let y = -y;
        eprintln!(
            "creating entity {} at {},{} with sprite {}",
            tile.r#type, x, y, tile.id
        );
        let entity = match &*tile.r#type {
            AIR_TILE_TYPE => world.create_entity(),
            PLATFORM_TILE_TYPE => world::create_platform(world),
            PLAYER_TILE_TYPE => world::create_player(world).with(CameraTarget {
                offset: Vector3::new(0.0, 0.0, 1.0),
                target_rotation: UnitQuaternion::identity(),
            }),
            other => panic!("unknown tile type {:?}", other),
        };
        entity
            .with(
                Transform::default()
                    .append_translation([x as f32 * 16.0, y as f32 * 16.0, 0.0].into())
                    .clone(),
            )
            .with(sprite_render)
            .build();
    });
}
