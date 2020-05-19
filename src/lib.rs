#![allow(dead_code, unused_imports, unused_variables)]
use amethyst::{
    audio::{AudioBundle, DjSystemDesc},
    core::transform::TransformBundle,
    input::{InputBundle, StringBindings},
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    ui::{RenderUi, UiBundle},
    utils::application_root_dir,
};

mod audio;
mod collisions;
mod game;
mod systems;
mod world;

use crate::{audio::Music, game::Game};

pub fn run() -> amethyst::Result<()> {
    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("config").join("display.ron");
    let bindings_path = app_root.join("config").join("bindings.ron");

    let input_bundle =
        InputBundle::<StringBindings>::new().with_bindings_from_file(bindings_path)?;

    let game_data = GameDataBuilder::default()
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.0, 0.0, 0.0, 1.0]),
                )
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderUi::default()),
        )?
        .with_bundle(TransformBundle::new())?
        .with_bundle(input_bundle)?
        .with_bundle(UiBundle::<StringBindings>::new())?
        // .with_bundle(AudioBundle::default())?
        //.with_bundle(PhysicsBundle::<f32, NPhysicsBackend>::new())?
        // .with(systems::PaddleSystem, "paddle_system", &["input_system"])
        // .with(systems::MoveBallsSystem, "ball_system", &[])
        // .with(
        //     systems::BounceSystem,
        //     "collision_system",
        //     &["paddle_system", "ball_system"],
        // )
        // .with(systems::WinnerSystem, "winner_system", &["ball_system"])
        // .with_system_desc(
        //     DjSystemDesc::new(|music: &mut Music| music.music.next()),
        //     "dj_system",
        //     &[],
        // )
        .with(
            systems::Ncollide2dSyncPresencesSystem::default(),
            "ncollide2d_sync_presence",
            &[],
        )
        .with(
            systems::Ncollide2dSyncTransformsSystem::default(),
            "ncollide2d_sync_transform",
            &[],
        )
        .with(
            systems::Ncollide2dUpdateWorldSystem::default(),
            "ncollide2d_update_world",
            &["ncollide2d_sync_presence", "ncollide2d_sync_transform"],
        )
        .with(
            systems::MovePlayerSystem,
            "move_player",
            &["ncollide2d_update_world"],
        )
        .with(
            systems::GravitySystem,
            "gravity",
            &["ncollide2d_update_world", "move_player"],
        )
        .with(
            systems::ApplyVelocity,
            "apply_velocity",
            &["ncollide2d_update_world", "move_player", "gravity"],
        );

    let assets_dir = app_root.join("assets");
    let mut game = Application::new(assets_dir, Game::default(), game_data)?;
    game.run();

    Ok(())
}
