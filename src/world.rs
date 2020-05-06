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
use amethyst_physics::objects::CollisionGroup;
use hibitset::BitSet;
use nalgebra::{Isometry2, Isometry3, Translation2, Translation3, UnitComplex, Vector2, Vector3};
use ncollide2d::{
    pipeline::{
        object::{CollisionObject, CollisionObjectSlabHandle},
        world::CollisionWorld,
        CollisionGroups, GeometricQueryType,
    },
    shape::Cuboid,
    shape::ShapeHandle,
};
use std::collections::BTreeMap;

use crate::collisions::*;

const PLATFORM_COLLISION_GROUP: usize = 1;
const PLAYER_COLLISION_GROUP: usize = 2;

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
    world.create_entity().with(Player).with(CollisionPresence {
        shape: ShapeHandle::new(Cuboid::new([8.0, 8.0].into())),
        collision_groups,
        query_type: GeometricQueryType::Contacts(0.0, 0.0),
    })
}
