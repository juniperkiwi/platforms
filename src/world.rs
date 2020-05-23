use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    core::{timing::Time, Transform},
    ecs::{
        prelude::*,
        world::{EntitiesRes, Index},
    },
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture},
    shred::DynamicSystemData,
    shrev::EventChannel,
    ui::{Anchor, TtfFormat, UiText, UiTransform},
};
use nalgebra::{Isometry2, Isometry3, Translation2, Translation3, UnitComplex, Vector2, Vector3};
use ncollide2d::{
    pipeline::{CollisionGroups, GeometricQueryType},
    shape::{Cuboid, ShapeHandle},
};

use crate::{
    collisions::components::{CollisionPresence, HasGravity},
    systems::CameraFollowConstants,
};

const PLATFORM_COLLISION_GROUP: usize = 1;
const PLAYER_COLLISION_GROUP: usize = 2;

// pub const PLAYER_JUMP: f32 = 300.0;
// pub const GRAVITY_ACCEL: f32 = 15.0;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ConstantsConfig {
    pub player_horizontal_velocity: f32,
    pub player_jump: f32,
    pub gravity_accel: f32,
    pub camera_follow: CameraFollowConstants,
}

#[derive(Copy, Clone, Default)]
pub struct Platform;

impl Component for Platform {
    type Storage = NullStorage<Self>;
}

#[derive(Copy, Clone, Default)]
pub struct Player;

impl Component for Player {
    type Storage = NullStorage<Self>;
}

#[derive(Copy, Clone)]
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

impl Component for Velocity {
    type Storage = DenseVecStorage<Self>;
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
        .with(Player)
        .with(CollisionPresence {
            shape: ShapeHandle::new(Cuboid::new([8.0, 8.0].into())),
            collision_groups,
            query_type: GeometricQueryType::Contacts(20.0, 0.0),
        })
        .with(HasGravity)
        .with(Velocity::default())
}
