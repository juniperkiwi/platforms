use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    core::Transform,
    ecs::prelude::{Component, DenseVecStorage, Entity},
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture},
    ui::{Anchor, TtfFormat, UiText, UiTransform},
};
use nalgebra::{UnitQuaternion, Vector3};

use crate::{
    systems::{CameraTarget, CameraVelocity, TrackingCamera},
    world,
};

pub const ARENA_HEIGHT: f32 = 100.0;
pub const ARENA_WIDTH: f32 = 100.0;

pub const PADDLE_HEIGHT: f32 = 16.0;
pub const PADDLE_WIDTH: f32 = 4.0;

pub const BALL_VELOCITY_X: f32 = 75.0;
pub const BALL_VELOCITY_Y: f32 = 50.0;
pub const BALL_RADIUS: f32 = 2.0;

pub const PADDLE_VELOCITY: f32 = 80.0;

fn initialize_camera(world: &mut World) {
    let mut transform = Transform::default();
    // transform.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 1.0);
    transform.set_translation_xyz(16.0, 32.0, 1.0);

    world
        .create_entity()
        .with(Camera::standard_2d(ARENA_WIDTH, ARENA_HEIGHT))
        .with(transform)
        .with(CameraVelocity::default())
        .with(TrackingCamera)
        .build();
}

fn initialize_box(world: &mut World, sprite_render: SpriteRender) {
    (0..20).for_each(|i| {
        world::create_platform(world)
            .with(
                Transform::default()
                    .append_translation([i as f32 * 16.0, 0.0, 0.0].into())
                    .clone(),
            )
            .with(sprite_render.clone())
            .build();
    });
    (0..20).for_each(|i| {
        world::create_platform(world)
            .with(
                Transform::default()
                    .append_translation([i as f32 * 16.0, 16.0 * 19.0, 0.0].into())
                    .clone(),
            )
            .with(sprite_render.clone())
            .build();
    });
    (0..20).for_each(|i| {
        world::create_platform(world)
            .with(
                Transform::default()
                    .append_translation([0.0, i as f32 * 16.0, 0.0].into())
                    .clone(),
            )
            .with(sprite_render.clone())
            .build();
    });
    (0..20).for_each(|i| {
        world::create_platform(world)
            .with(
                Transform::default()
                    .append_translation([16.0 * 19.0, i as f32 * 16.0, 0.0].into())
                    .clone(),
            )
            .with(sprite_render.clone())
            .build();
    });
}

fn initialize_single(world: &mut World, sprite_render: SpriteRender) {
    world::create_platform(world)
        .with(
            Transform::default()
                .append_translation([40.0, 20.0, 0.0].into())
                .clone(),
        )
        .with(sprite_render.clone())
        .build();
}

fn initialize_stairs(world: &mut World, sprite_render: SpriteRender) {
    (0..20).for_each(|i| {
        world::create_platform(world)
            .with(
                Transform::default()
                    .append_translation([i as f32 * 16.0, i as f32 * 16.0, 0.0].into())
                    .clone(),
            )
            .with(sprite_render.clone())
            .build();
    });
}

pub fn initialize_platforms(world: &mut World, sprite_sheet: Handle<SpriteSheet>) {
    let sprite_render = SpriteRender {
        sprite_sheet,
        sprite_number: 3,
    };

    world.register::<world::Platform>();
    initialize_box(world, sprite_render.clone());
    initialize_single(world, sprite_render.clone());
    initialize_stairs(world, sprite_render.clone());
}

fn initialize_player(world: &mut World, sprite_sheet: Handle<SpriteSheet>) {
    let sprite_render = SpriteRender {
        sprite_sheet,
        sprite_number: 4,
    };

    world.register::<world::Player>();

    world::create_player(world)
        .with(
            Transform::default()
                .append_translation(Vector3::new(50.0, 150.0, 0.0))
                .clone(),
        )
        .with(CameraTarget {
            offset: Vector3::new(0.0, 0.0, 1.0),
            target_rotation: UnitQuaternion::identity(),
        })
        .with(sprite_render.clone())
        .build();
}

fn initialize_ball(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>) {
    let mut local_transform = Transform::default();
    local_transform.set_translation_xyz(ARENA_WIDTH / 2.0, ARENA_HEIGHT / 2.0, 0.0);

    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 1,
    };

    world
        .create_entity()
        .with(sprite_render)
        .with(Ball {
            radius: BALL_RADIUS,
            velocity: [BALL_VELOCITY_X, BALL_VELOCITY_Y],
        })
        .with(local_transform)
        .build();
}

fn initialize_scoreboard(world: &mut World) {
    let font = world.read_resource::<Loader>().load(
        "font/square.ttf",
        TtfFormat,
        (),
        &world.read_resource(),
    );

    let p1_transform = UiTransform::new(
        "P1".to_string(),
        Anchor::TopMiddle,
        Anchor::TopMiddle,
        -50.0,
        -50.0,
        1.0,
        200.0,
        50.0,
    );
    let p2_transform = UiTransform::new(
        "P2".to_string(),
        Anchor::TopMiddle,
        Anchor::TopMiddle,
        50.0,
        -50.0,
        1.0,
        200.0,
        50.0,
    );
    let p1_score = world
        .create_entity()
        .with(p1_transform)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            [1.0, 1.0, 1.0, 1.0],
            50.0,
        ))
        .build();
    let p2_score = world
        .create_entity()
        .with(p2_transform)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            [1.0, 1.0, 1.0, 1.0],
            50.0,
        ))
        .build();

    world.insert(ScoreText { p1_score, p2_score })
}

fn load_sprite_sheet(world: &mut World) -> Handle<SpriteSheet> {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            "texture/mountain_base_tileset.png",
            ImageFormat::default(),
            (),
            &texture_storage,
        )
    };
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();

    loader.load(
        "texture/mountain_base_tileset.ron",
        SpriteSheetFormat(texture_handle),
        (),
        &sprite_sheet_store,
    )
}

#[derive(Default)]
pub struct Game {
    ball_spawn_timer: Option<f32>,
    sprite_sheet: Option<Handle<SpriteSheet>>,
}

impl SimpleState for Game {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        let sprite_sheet = load_sprite_sheet(world);

        initialize_camera(world);
        // initialize_scoreboard(world);
        // initialize_audio(world);
        initialize_platforms(world, sprite_sheet.clone());
        initialize_player(world, sprite_sheet.clone());

        self.sprite_sheet.replace(sprite_sheet);

        self.ball_spawn_timer = Some(1.0);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        // if let Some(mut timer) = self.ball_spawn_timer.take() {
        //     {
        //         let time = data.world.fetch::<Time>();
        //         timer -= time.delta_seconds();
        //     }
        //     if timer <= 0.0 {
        //         initialize_ball(data.world, self.sprite_sheet.clone().unwrap());
        //     } else {
        //         self.ball_spawn_timer.replace(timer);
        //     }
        // }
        Trans::None
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}
pub struct Paddle {
    pub side: Side,
    pub width: f32,
    pub height: f32,
}

impl Paddle {
    fn new(side: Side) -> Paddle {
        Paddle {
            side,
            width: PADDLE_WIDTH,
            height: PADDLE_HEIGHT,
        }
    }
}

impl Component for Paddle {
    type Storage = DenseVecStorage<Self>;
}

pub struct Ball {
    pub velocity: [f32; 2],
    pub radius: f32,
}

impl Component for Ball {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default)]
pub struct Scoreboard {
    pub score_left: i32,
    pub score_right: i32,
}

pub struct ScoreText {
    pub p1_score: Entity,
    pub p2_score: Entity,
}
