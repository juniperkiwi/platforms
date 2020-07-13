use amethyst::ecs::prelude::*;
use nalgebra::Vector2;
use ncollide2d::{
    pipeline::{CollisionGroups, GeometricQueryType},
    shape::{Cuboid, ShapeHandle},
};
use specs_derive::Component;

use crate::{
    collisions::components::{CollisionPresence, HasGravity},
    systems::{CameraFollowConstants, PlayerMovementConstants},
};
use std::path::PathBuf;

const PLATFORM_COLLISION_GROUP: usize = 1;
const PLAYER_COLLISION_GROUP: usize = 2;

// pub const PLAYER_JUMP: f32 = 300.0;
// pub const GRAVITY_ACCEL: f32 = 15.0;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ConstantsConfig {
    pub player: PlayerMovementConstants,
    pub gravity_accel: f32,
    pub camera_follow: CameraFollowConstants,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MapsConfig {
    pub default: PathBuf,
}

#[derive(Copy, Clone, Default, Component)]
#[storage(NullStorage)]
pub struct Platform;

#[derive(Copy, Clone, Component, Default)]
#[storage(DenseVecStorage)]
pub struct Player {
    pub air_boost: f32,
}

#[derive(Copy, Clone, Component)]
#[storage(DenseVecStorage)]
pub struct Velocity {
    pub(crate) intended: Vector2<f32>,
}
impl Default for Velocity {
    fn default() -> Self {
        Velocity {
            intended: [0.0; 2].into(),
        }
    }
}

pub fn create_platform(world: &mut World) -> EntityBuilder {
    let mut collision_groups = CollisionGroups::new()
        .with_membership(&[PLATFORM_COLLISION_GROUP])
        .with_whitelist(&[PLAYER_COLLISION_GROUP]);
    collision_groups.disable_self_interaction();
    world
        .create_entity()
        .with(Platform)
        .with(CollisionPresence {
            shape: ShapeHandle::new(Cuboid::new([8.0, 8.0].into())),
            collision_groups,
            query_type: GeometricQueryType::Contacts(0.0, 0.0),
        })
}

pub fn create_player(world: &mut World) -> EntityBuilder {
    let mut collision_groups = CollisionGroups::new()
        .with_membership(&[PLAYER_COLLISION_GROUP])
        .with_whitelist(&[PLATFORM_COLLISION_GROUP]);
    collision_groups.disable_self_interaction();
    world
        .create_entity()
        .with(Player::default())
        .with(CollisionPresence {
            shape: ShapeHandle::new(Cuboid::new([8.0, 8.0].into())),
            collision_groups,
            query_type: GeometricQueryType::Contacts(20.0, 0.0),
        })
        .with(HasGravity)
        .with(Velocity::default())
}
