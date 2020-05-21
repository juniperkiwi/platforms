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
                    //let rotation_accel = target_rotation.nlerp(transform.rotation(), time.delta_seconds() * constants.camera_rotation_modifier);
                    let translation_accel = ((target_translation - transform.translation())
                        * constants.camera_translation_modifier1)
                        .map(|f| {
                            let f2 = f.powf(constants.camera_translation_modifier2);
                            if f2.is_finite() {
                                f2
                            } else {
                                f
                            }
                        })
                        * time.delta_seconds();

                    //velocity.rotation = velocity.rotation + rotation_accel;
                    velocity.translation += translation_accel;
                    velocity.translation = velocity.translation.lerp(
                        &(velocity.translation
                            * time.delta_seconds()
                            * constants.camera_translation_dampener_mul),
                        time.delta_seconds(),
                    );
                    velocity.translation = velocity.translation.lerp(
                        &velocity.translation.zip_map(
                            &(transform.translation() - target_translation),
                            |f, dist| {
                                let time_to_reach = (dist / f).abs();
                                let time2 = time_to_reach.min(
                                    constants
                                        .camera_translation_time_to_reach_before_dampening_secs,
                                );
                                let inverted_time = 1.0 / time2;
                                f * (1.0
                                    - (inverted_time
                                        * constants
                                            .camera_translation_dampener_distance_modifier_for_mul)
                                        .min(constants.camera_translation_max_damp2))
                            },
                        ),
                        time.delta_seconds(),
                    );
                    velocity.translation = velocity.translation.lerp(
                        &velocity.translation.map(|f| {
                            f.signum() * f.abs().powf(constants.camera_translation_dampener_pow)
                        }),
                        time.delta_seconds(),
                    );
                    //transform.set_rotation(transform.rotation() + velocity.rotation * time.delta_seconds());
                    transform.set_translation(transform.translation().zip_zip_map(
                        &target_translation,
                        &(velocity.translation * time.delta_seconds()),
                        |current, target, movement| {
                            let distance = target - current;
                            if distance.signum() == movement.signum() && movement.abs() > distance.abs() {
                                println!(
                                    "going directly to target as {}.signum() \
                                    == {}.signum() == {} and {} > {}",
                                    distance,
                                    movement,
                                    distance.signum(),
                                    movement,
                                    distance
                                );
                                target
                            } else {
                                current + movement
                            }
                        },
                    ));
                }
                None => {
                    transform.set_translation(target_translation);
                    transform.set_rotation(target_rotation);
                }
            };
        }
    }
}
