use crate::world::*;
use alga::linear::AffineTransformation;
use amethyst::{
    core::{timing::Time, SystemDesc, Transform},
    derive::SystemDesc,
    ecs::prelude::*,
    input::{InputHandler, StringBindings},
};
use log::debug;
use nalgebra::{
    Isometry2, Translation2, Translation3, Unit, UnitComplex, UnitQuaternion, Vector2, Vector3,
};
use ncollide2d::{
    bounding_volume::bounding_volume::BoundingVolume,
    pipeline::{
        narrow_phase::ContactAlgorithm, object::CollisionObjectSlabHandle, CollisionObject,
        CollisionWorld,
    },
    query::{Contact, ContactManifold},
    shape::Shape,
};

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
    fn prepend_movement(&self, direction: Unit<Vector2<f32>>, distance: f32) -> Self;
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

    fn prepend_movement(&self, direction: Unit<Vector2<f32>>, distance: f32) -> Self {
        Isometry2::from_parts(
            self.translation
                .prepend_translation(&Translation2::from(direction.as_ref() * distance)),
            self.rotation,
        )
    }
}
