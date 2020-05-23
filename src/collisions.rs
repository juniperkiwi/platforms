use std::collections::BTreeMap;

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
use hibitset::BitSet;
use nalgebra::{
    Isometry2, Isometry3, Translation2, Translation3, UnitComplex, UnitQuaternion, Vector2, Vector3,
};
use ncollide2d::{
    pipeline::{
        object::{CollisionObject, CollisionObjectSlabHandle},
        world::CollisionWorld,
        CollisionGroups, GeometricQueryType,
    },
    shape::ShapeHandle,
};

pub mod components;
pub mod prelude;
pub mod resolution_utils;
