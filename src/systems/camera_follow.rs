use crate::{
    collisions::{
        CollisionPresence, IsometryExt, Ncollide2dHandle, Ncollide2dWorld, TransformExt, *,
    },
    game::{Paddle, Side, ARENA_HEIGHT, PADDLE_HEIGHT, PADDLE_VELOCITY},
    world::*,
};
use alga::linear::AffineTransformation;
use amethyst::{
    core::{timing::Time, SystemDesc, Transform},
    derive::SystemDesc,
    ecs::prelude::*,
    input::{InputHandler, StringBindings},
};
use log::debug;
use nalgebra::{Isometry2, Translation2, Unit, UnitQuaternion, Vector2, Vector3};
use ncollide2d::{
    bounding_volume::bounding_volume::BoundingVolume,
    pipeline::{
        narrow_phase::ContactAlgorithm, object::CollisionObjectSlabHandle, CollisionObject,
        CollisionWorld,
    },
    query::{Contact, ContactManifold},
    shape::Shape,
};

#[derive(Default)]
pub struct TrackingCamera;

impl Component for TrackingCamera {
    type Storage = NullStorage<Self>;
}

#[derive(Clone)]
pub struct CameraTarget {
    pub offset: Vector3<f32>,
    pub target_rotation: UnitQuaternion<f32>,
}

impl Component for CameraTarget {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Clone)]
pub struct CameraVelocity {
    pub translation: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
}

impl Component for CameraVelocity {
    type Storage = DenseVecStorage<Self>;
}
impl Default for CameraVelocity {
    fn default() -> Self {
        CameraVelocity {
            translation: Vector3::zeros(),
            rotation: UnitQuaternion::identity(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CameraFollowConstants {
    time_to_target: f32,
    time_to_velocity: f32,
    lerp_multiplier: f32,
    min_velocity: f32,
    max_velocity: f32,
}

#[derive(SystemDesc)]
pub struct CameraTrackTargetSystem;

impl<'s> System<'s> for CameraTrackTargetSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, CameraVelocity>,
        ReadStorage<'s, TrackingCamera>,
        ReadStorage<'s, CameraTarget>,
        Read<'s, Time>,
        ReadExpect<'s, ConstantsConfig>,
    );
    fn run(
        &mut self,
        (mut transforms, mut camera_velocities, tracking_cameras, camera_targets, time, constants): Self::SystemData,
    ) {
        let constants = &constants.camera_follow;
        let mut target_data = None;
        for (transform, tracking) in (&transforms, &camera_targets).join() {
            assert!(target_data.is_none(), "duplicate camera tracking targets");
            target_data.replace((transform.clone(), tracking.clone()));
        }
        let (target_translation, target_rotation) = {
            let (mut transform, targeting_info) = match target_data {
                Some(v) => v,
                None => return,
            };
            transform.prepend_translation(targeting_info.offset);
            (*transform.translation(), targeting_info.target_rotation)
        };

        for (transform, velocity, _) in (
            &mut transforms,
            (&mut camera_velocities).maybe(),
            &tracking_cameras,
        )
            .join()
        {
            match velocity {
                Some(velocity) => {
                    let here = *transform.translation();
                    // let velocity = &mut velocity.translation;
                    let delta_t = time.delta_seconds();
                    let distance = target_translation - here;
                    // let target_velocity = distance / constants.time_to_target;
                    // *velocity =
                    // velocity.lerp(&target_velocity, delta_t /
                    // constants.time_to_velocity);
                    let mut velocity = constants.lerp_multiplier * distance;
                    if velocity.magnitude() > constants.max_velocity {
                        velocity = velocity.normalize() * constants.max_velocity;
                    } else if velocity.magnitude() < constants.min_velocity {
                        velocity =
                            velocity.normalize() * constants.min_velocity.min(distance.magnitude());
                    }
                    transform.set_translation(here + (delta_t * velocity));
                }
                None => {
                    transform.set_translation(target_translation);
                    transform.set_rotation(target_rotation);
                }
            };
        }
    }
}

trait FloatExt {
    fn finite_or(self, v: Self) -> Self;
}
impl FloatExt for f32 {
    fn finite_or(self, v: Self) -> Self {
        if self.is_finite() {
            self
        } else {
            v
        }
    }
}
