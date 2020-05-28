use amethyst::{
    core::{timing::Time, Transform},
    derive::SystemDesc,
    ecs::prelude::*,
};
use nalgebra::{UnitQuaternion, Vector3};

use crate::world::*;

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
}

impl Component for CameraVelocity {
    type Storage = DenseVecStorage<Self>;
}
impl Default for CameraVelocity {
    fn default() -> Self {
        CameraVelocity {
            translation: Vector3::zeros(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CameraFollowConstants {
    time_to_target: f32,
    smoothing_factor: f32,
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
                    // Credit for the algorithm here to
                    // https://github.com/azriel91/autexousious/blob/0.19.0/crate/camera_play/src/system/camera_velocity_system.rs
                    let here = *transform.translation();
                    let delta_t = time.delta_seconds();
                    let distance = target_translation - here;
                    let target_velocity = distance / constants.time_to_target;
                    velocity.translation = velocity
                        .translation
                        .lerp(&target_velocity, constants.smoothing_factor * delta_t);
                    if velocity.translation.magnitude() * delta_t > distance.magnitude() {
                        velocity.translation = Vector3::zeros();
                        transform.set_translation(target_translation);
                    } else {
                        transform.set_translation(here + (delta_t * velocity.translation));
                    }
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
