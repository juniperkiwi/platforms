use amethyst::{
    core::{timing::Time, Transform},
    derive::SystemDesc,
    ecs::prelude::*,
};
use log::debug;
use nalgebra::{Unit, Vector2, Vector3};

use super::Ncollide2dWorld;
use crate::{
    collisions::{
        components::{CollisionPresence, Ncollide2dHandle},
        prelude::{IsometryExt, TransformExt},
        resolution_utils::contact_in_direction_with_shape,
    },
    world::*,
};

#[derive(SystemDesc)]
pub struct ApplyVelocity;

pub const MARGIN_DISTANCE: f32 = 0.01;

impl<'s> System<'s> for ApplyVelocity {
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Velocity>,
        ReadStorage<'s, CollisionPresence>,
        ReadStorage<'s, Ncollide2dHandle>,
        Read<'s, Ncollide2dWorld>,
        Read<'s, Time>,
    );
    fn run(
        &mut self,
        (mut transforms, mut velocities, presences, handles, ncollide_world, time): Self::SystemData,
    ) {
        let ncollide_world = &ncollide_world.world;
        let delta_seconds = time.delta_seconds();
        for (transform, velocity, presence, handle) in
            (&mut transforms, &mut velocities, &presences, &handles).join()
        {
            let mut isometry = transform.to_2d_isometry();
            let shape = &*presence.shape;
            let velocity = &mut velocity.intended;
            let handle = handle.0;
            let mut direction = Unit::<Vector2<f32>>::new_normalize(*velocity);
            let mut maximum_distance = velocity.magnitude() * delta_seconds;

            let shape = &*presence.shape;
            let all_clear = ncollide_world
                .sweep_test(
                    shape,
                    &isometry,
                    &direction,
                    maximum_distance,
                    &presence.collision_groups,
                )
                .next()
                .is_none();
            if all_clear {
                transform.prepend_translation(xy_with_zero_z(*velocity * delta_seconds));
                continue;
            }
            debug!(
                "---- calculating collisions for object at {},{} with velocity {},{}",
                isometry.translation.x, isometry.translation.y, velocity.x, velocity.y
            );
            let mut remaining_time = delta_seconds;
            let mut iterations_left = 5;
            let all_clear = loop {
                let sweep = ncollide_world.sweep_test(
                    shape,
                    &isometry,
                    &direction,
                    maximum_distance,
                    &presence.collision_groups,
                );
                let nearest = sweep
                    .filter_map(|(obj, toi)| {
                        let effected_by_toi = isometry.prepend_movement(direction, toi.toi);
                        let obj = ncollide_world.objects.get(obj).unwrap();

                        let contact2 = contact_in_direction_with_shape(
                            &effected_by_toi,
                            shape,
                            obj,
                            direction,
                        )?;

                        Some((obj, toi, contact2))
                    })
                    .min_by(|(_, toi1, _), (_, toi2, _)| toi1.toi.partial_cmp(&toi2.toi).unwrap());

                let (obj, toi, contact_at_depth) = match nearest {
                    Some(v) => v,
                    None => break true,
                };

                debug!(
                    "found a collision for {},{} moving {} in {},{}! Collision is with {},{} with normal1: {},{}, normal2: {},{} (full: {:?})",
                    transform.translation().x,
                    transform.translation().y,
                    maximum_distance,
                    direction.as_ref().x,
                    direction.as_ref().y,
                    obj.position().translation.x,
                    obj.position().translation.y,
                    toi.normal1.x,
                    toi.normal1.y,
                    toi.normal2.x,
                    toi.normal2.y,
                    toi,
                );

                transform.prepend_translation(xy_with_zero_z(toi.toi * direction.as_ref()));
                remaining_time -= toi.toi / velocity.magnitude();
                iterations_left -= 1;
                // kill velocity towards the obstacle.
                let old_vel = *velocity;
                *velocity -= velocity.dot(contact_at_depth.normal.as_ref())
                    * contact_at_depth.normal.as_ref();
                debug!(
                    "velocity change: {},{} -> {},{}",
                    old_vel.x, old_vel.y, velocity.x, velocity.y
                );
                direction = Unit::new_normalize(*velocity); // note: will be NaN if velocity == 0.0.
                isometry = transform.to_2d_isometry();
                maximum_distance = velocity.magnitude() * delta_seconds;
                if remaining_time <= 0.0 || iterations_left == 0 || maximum_distance == 0.0 {
                    break false;
                }
                debug!(
                    "more movement left! new direction is {},{}",
                    direction.as_ref().x,
                    direction.as_ref().y
                );
            };
            // do the last bit of movement if we stopped b/c of remaining_time
            // or iterations_left.
            if all_clear {
                transform.prepend_translation(xy_with_zero_z(*velocity * remaining_time));
            }
        }
    }
}

fn xy_with_zero_z(t: Vector2<f32>) -> Vector3<f32> {
    [t.x, t.y, 0.0].into()
}
