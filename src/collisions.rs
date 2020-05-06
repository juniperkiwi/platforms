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
use amethyst_physics::objects::CollisionGroup;
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

pub struct Ncollide2dWorld {
    pub world: CollisionWorld<f32, Entity>,
}

impl Default for Ncollide2dWorld {
    fn default() -> Self {
        Ncollide2dWorld {
            world: CollisionWorld::new(0.02),
        }
    }
}

#[derive(Clone)]
pub struct CollisionPresence {
    pub shape: ShapeHandle<f32>,
    pub collision_groups: CollisionGroups,
    pub query_type: GeometricQueryType<f32>,
}

impl Component for CollisionPresence {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

pub struct Ncollide2dHandle(pub(crate) CollisionObjectSlabHandle);

impl Component for Ncollide2dHandle {
    type Storage = VecStorage<Self>;
}

pub trait TransformExt {
    fn to_2d_isometry(&self) -> Isometry2<f32>;
}
impl TransformExt for Transform {
    fn to_2d_isometry(&self) -> ncollide2d::math::Isometry<f32> {
        let isometry = self.isometry();

        let translation = {
            let x = isometry.translation.vector.x;
            let y = isometry.translation.vector.y;
            // ignore translation z

            Translation2::new(x, y)
        };
        let rotation = match isometry.rotation.axis_angle() {
            Some((axis, angle)) => {
                assert!(axis.into_inner() == Vector3::z());
                UnitComplex::new(angle)
            }
            None => UnitComplex::identity(),
        };

        Isometry2::from_parts(translation, rotation)
    }
}
pub trait IsometryExt {
    fn to_transform(&self) -> Transform;
}

impl IsometryExt for ncollide2d::math::Isometry<f32> {
    fn to_transform(&self) -> Transform {
        let translation = {
            let x = self.translation.vector.x;
            let y = self.translation.vector.y;

            Translation3::new(x, y, 0.0)
        };
        let rotation = UnitQuaternion::from_axis_angle(&Vector3::z_axis(), self.rotation.angle());

        Transform::new(translation, rotation, [1.0, 1.0, 1.0].into())
    }
}
