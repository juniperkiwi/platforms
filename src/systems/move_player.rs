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
use nalgebra::{Isometry2, Translation2, Unit, Vector2, Vector3};
use ncollide2d::{
    bounding_volume::bounding_volume::BoundingVolume,
    pipeline::{
        narrow_phase::ContactAlgorithm, object::CollisionObjectSlabHandle, CollisionObject,
        CollisionWorld,
    },
    query::{Contact, ContactManifold},
    shape::Shape,
};

#[derive(SystemDesc)]
pub struct MovePlayerSystem;

impl<'s> System<'s> for MovePlayerSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Velocity>,
        ReadStorage<'s, Ncollide2dHandle>,
        ReadStorage<'s, Player>,
        Read<'s, InputHandler<StringBindings>>,
        Read<'s, Ncollide2dWorld>,
        Read<'s, Time>,
        ReadExpect<'s, ConstantsConfig>,
    );
    fn run(
        &mut self,
        (mut transforms, mut velocities, handles, players, input, ncollide_world, time, constants): Self::SystemData,
    ) {
        let ncollide_world = &ncollide_world.world;
        for (transform, velocity, handle, _) in
            (&mut transforms, &mut velocities, &handles, &players).join()
        {
            let lr = input.axis_value("left_right");
            let jump = input.action_is_down("jump");
            if let Some(lr) = lr {
                velocity.intended.x = lr as f32 * constants.player_horizontal_velocity;
            }
            if jump.unwrap_or(false) {
                if on_floor(ncollide_world, handle.0) {
                    debug!("jumping from floor!");
                    velocity.intended.y += constants.player_jump;
                } else {
                    debug!("jumping but not on floor");
                }
            }
        }
    }
}

const DIRECTION_TEST_DELTA: f32 = 0.001;

fn contact_in_direction<T>(
    obj1: &CollisionObject<f32, T>,
    obj2: &CollisionObject<f32, T>,
    direction: Unit<Vector2<f32>>,
) -> Option<Contact<f32>> {
    let isometry = obj1.position();
    let in_depth = isometry.prepend_movement(direction, DIRECTION_TEST_DELTA);
    let contact1 = ncollide2d::query::contact(
        obj1.position(),
        &**obj1.shape(),
        obj2.position(),
        &**obj2.shape(),
        0.0,
    )?;
    let contact2 = ncollide2d::query::contact(
        &in_depth,
        &**obj1.shape(),
        obj2.position(),
        &**obj2.shape(),
        0.0,
    )?;
    if contact2.depth > contact1.depth {
        Some(contact2)
    } else {
        None
    }
}
fn contact_in_direction_with_shape<T>(
    isometry: &Isometry2<f32>,
    shape: &dyn Shape<f32>,
    obj2: &CollisionObject<f32, T>,
    direction: Unit<Vector2<f32>>,
) -> Option<Contact<f32>> {
    let in_depth = isometry.prepend_movement(direction, DIRECTION_TEST_DELTA);
    let contact1 =
        ncollide2d::query::contact(isometry, shape, obj2.position(), &**obj2.shape(), 0.0)?;
    let contact2 =
        ncollide2d::query::contact(&in_depth, shape, obj2.position(), &**obj2.shape(), 0.0)?;
    if contact2.depth > contact1.depth {
        Some(contact2)
    } else {
        None
    }
}

fn on_floor<T>(ncollide_world: &CollisionWorld<f32, T>, handle: CollisionObjectSlabHandle) -> bool {
    ncollide_world
        .contacts_with(handle, true)
        .into_iter()
        .flat_map(|v| v)
        .any(|(handle1, handle2, _algo, manifold)| {
            contact_in_direction(
                ncollide_world.objects.get(handle1).unwrap(),
                ncollide_world.objects.get(handle2).unwrap(),
                -Vector2::y_axis(),
            )
            .is_some()
        })
}

#[derive(SystemDesc)]
pub struct GravitySystem;

impl<'s> System<'s> for GravitySystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Velocity>,
        ReadStorage<'s, Ncollide2dHandle>,
        Read<'s, Ncollide2dWorld>,
        Read<'s, Time>,
        ReadExpect<'s, ConstantsConfig>,
    );
    fn run(
        &mut self,
        (mut transforms, mut velocities, handles, ncollide_world, time, constants): Self::SystemData,
    ) {
        let ncollide_world = &ncollide_world.world;
        for (transform, velocity, handle) in (&mut transforms, &mut velocities, &handles).join() {
            if on_floor(ncollide_world, handle.0) {
                if velocity.intended.y < 0.0 {
                    velocity.intended.y = 0.0;
                    debug!("gravity: on floor");
                }
            } else {
                velocity.intended.y -= constants.gravity_accel * time.delta_seconds();
            }
        }
    }
}

trait Isometry2Ext {
    fn prepend_movement(&self, direction: Unit<Vector2<f32>>, distance: f32) -> Self;
}

impl Isometry2Ext for Isometry2<f32> {
    fn prepend_movement(&self, direction: Unit<Vector2<f32>>, distance: f32) -> Self {
        Isometry2::from_parts(
            self.translation
                .prepend_translation(&Translation2::from(direction.as_ref() * distance)),
            self.rotation,
        )
    }
}

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
