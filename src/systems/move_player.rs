use crate::{
    collisions::{
        CollisionPresence, IsometryExt, Ncollide2dHandle, Ncollide2dWorld, TransformExt, *,
    },
    game::{Paddle, Side, ARENA_HEIGHT, PADDLE_HEIGHT, PADDLE_VELOCITY},
    world::*,
};
use amethyst::{
    core::{timing::Time, SystemDesc, Transform},
    derive::SystemDesc,
    ecs::prelude::*,
    input::{InputHandler, StringBindings},
};
use nalgebra::{Unit, Vector2, Vector3};
use ncollide2d::{
    pipeline::{narrow_phase::ContactAlgorithm, object::CollisionObjectSlabHandle},
    query::ContactManifold,
};

#[derive(SystemDesc)]
pub struct MovePlayerSystem;

macro_rules! format_contact {
    ($world:ident) => {
        (|(h1, h2, _algo, manifold): (
            CollisionObjectSlabHandle,
            CollisionObjectSlabHandle,
            &ContactAlgorithm<f32>,
            &ContactManifold<f32>,
        )| {
            let pos1 = $world.objects.get(h1).unwrap().position();
            let pos2 = $world.objects.get(h2).unwrap().position();
            match manifold.deepest_contact() {
                Some(deepest_contact) => format!(
                    "({},{} <-> {},{} @ normal {},{})",
                    pos1.translation.x,
                    pos1.translation.y,
                    pos2.translation.x,
                    pos2.translation.y,
                    deepest_contact.contact.normal.x,
                    deepest_contact.contact.normal.y
                ),
                None => format!(
                    "({},{} <-> {},{} | no_contact)",
                    pos1.translation.x, pos1.translation.y, pos2.translation.x, pos2.translation.y,
                ),
            }
        })
    };
}

impl<'s> System<'s> for MovePlayerSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Velocity>,
        ReadStorage<'s, Ncollide2dHandle>,
        ReadStorage<'s, Player>,
        Read<'s, InputHandler<StringBindings>>,
        Read<'s, Ncollide2dWorld>,
        Read<'s, Time>,
    );
    fn run(
        &mut self,
        (mut transforms, mut velocities, handles, players, input, ncollide_world, time): Self::SystemData,
    ) {
        let ncollide_world = &ncollide_world.world;
        for (transform, velocity, handle, _) in
            (&mut transforms, &mut velocities, &handles, &players).join()
        {
            let lr = input.axis_value("left_right");
            let jump = input.action_is_down("jump");
            if let Some(lr) = lr {
                // let scaled_amount = time.delta_seconds() * lr as f32 * PADDLE_VELOCITY;
                // let wall = ncollide_world
                //     .contacts_with(handle.0, true)
                //     .into_iter()
                //     .flat_map(|v| v)
                //     .find(|(_h1, _h2, _algo, manifold)| {
                //         let c = manifold.deepest_contact();
                //         c.map(|c| {
                //             c.contact.normal.as_ref().x.partial_cmp(&0.0)
                //                 == (lr as f32).partial_cmp(&0.0)
                //         })
                //         .unwrap_or(false)
                //     });
                // if scaled_amount != 0.0 || jump.unwrap_or(false) {
                //     eprintln!(
                //         "contacts: [{}]",
                //         ncollide_world
                //             .contacts_with(handle.0, true)
                //             .into_iter()
                //             .flat_map(|v| v)
                //             .map(format_contact!(ncollide_world))
                //             .collect::<Vec<_>>()
                //             .join(", ")
                //     );
                //     if let Some(wall) = wall {
                //         eprintln!("wall contact at {}", format_contact!(ncollide_world)(wall));
                //     }
                // }
                // if wall.is_none() {
                //     transform.prepend_translation(Vector3::new(scaled_amount, 0.0, 0.0));
                // }
                velocity.intended.x = lr as f32 * PADDLE_VELOCITY;
            }
            if jump.unwrap_or(false) {
                eprintln!("jumping!");
                let floor = ncollide_world
                    .contacts_with(handle.0, true)
                    .into_iter()
                    .flat_map(|v| v)
                    .filter_map(|(_h1, _h2, _algo, manifold)| manifold.deepest_contact())
                    .find(|c| c.contact.normal.as_ref().y < 0.0);
                if floor.is_some() {
                    eprintln!("also on floor!!");
                    velocity.intended.y += PLAYER_JUMP;
                }
            }
        }
    }
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
    );
    fn run(
        &mut self,
        (mut transforms, mut velocities, handles, ncollide_world, time): Self::SystemData,
    ) {
        let ncollide_world = &ncollide_world.world;
        for (transform, velocity, handle) in (&mut transforms, &mut velocities, &handles).join() {
            let floor = ncollide_world
                .contacts_with(handle.0, true)
                .into_iter()
                .flat_map(|v| v)
                .find(|(_h1, _h2, _algo, manifold)| {
                    let c = manifold.deepest_contact();
                    c.map(|c| c.contact.normal.as_ref().y < 0.0)
                        .unwrap_or(false)
                });
            // let ceiling = ncollide_world
            //     .contacts_with(handle.0, true)
            //     .into_iter()
            //     .flat_map(|v| v)
            //     .find(|(_h1, _h2, _algo, manifold)| {
            //         let c = manifold.deepest_contact();
            //         c.map(|c| c.contact.normal.as_ref().y > 0.0)
            //             .unwrap_or(false)
            //     });
            if floor.is_none() {
                velocity.intended.y -= GRAVITY_ACCEL;
            } else if velocity.intended.y < 0.0 {
                velocity.intended.y = 0.0;
                eprintln!(
                    "floor contact at {}",
                    format_contact!(ncollide_world)(floor.unwrap())
                );
            }
            // if ceiling.is_some() && velocity.intended.y > 0.0 {
            //     velocity.intended.y = -velocity.intended.y;
            //     eprintln!(
            //         "ceiling contact at {}",
            //         format_contact!(ncollide_world)(ceiling.unwrap())
            //     );
            // }
            // transform.prepend_translation(Vector3::new(
            //     0.0,
            //     velocity.intended.y * time.delta_seconds(),
            //     0.0,
            // ));
        }
    }
}

// #[derive(SystemDesc)]
// pub struct ApplyIntendedVelocityUpdatePredictedPositions;

// impl<'s> System<'s> for ApplyIntendedVelocityUpdatePredictedPositions {
//     type SystemData = (
//         ReadStorage<'s, Transform>,
//         WriteStorage<'s, Velocity>,
//         ReadStorage<'s, Ncollide2dHandle>,
//         Write<'s, Ncollide2dWorld>,
//         Read<'s, Time>,
//     );
//     fn run(&mut self, (transforms, velocities, handles, world, time): Self::SystemData) {

//     }
// }

#[derive(SystemDesc)]
pub struct ApplyVelocity;

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
        // const DIRECTIONS: &Vector2 = &[Vector2::x(), Vector2::y()];

        let ncollide_world = &ncollide_world.world;
        let delta_seconds = time.delta_seconds();
        for (transform, velocity, presence, handle) in
            (&mut transforms, &mut velocities, &presences, &handles).join()
        {
            let mut isometry = transform.to_2d_isometry();
            let velocity = &mut velocity.intended;
            let handle = handle.0;
            let mut direction = Unit::<Vector2<f32>>::new_normalize(*velocity);
            let mut maximum_distance = velocity.magnitude() * delta_seconds;

            let sweep = ncollide_world.sweep_test(
                &*presence.shape,
                &isometry,
                &direction,
                maximum_distance,
                &presence.collision_groups,
            );
            let mut nearest = sweep
                .filter(|(_, toi)| toi.normal1.as_ref().dot(&direction) > 0.0)
                .min_by(|(_, toi1), (_, toi2)| toi1.toi.partial_cmp(&toi2.toi).unwrap());
            let mut all_clear = nearest.is_none();
            if all_clear {
                transform.prepend_translation(xy_with_zero_z(*velocity * delta_seconds));
                continue;
            }
            let obj = ncollide_world
                .objects
                .get(nearest.as_ref().unwrap().0)
                .unwrap()
                .position();
            eprintln!(
                "found a collision for {},{} moving {} in {},{}! Collision is with {},{} with normal1: {},{}, normal2: {},{} (full: {:?})",
                transform.translation().x,
                transform.translation().y,
                maximum_distance,
                direction.as_ref().x,
                direction.as_ref().y,
                obj.translation.x,
                obj.translation.y,
                nearest.as_ref().unwrap().1.normal1.x,
                nearest.as_ref().unwrap().1.normal1.y,
                nearest.as_ref().unwrap().1.normal2.x,
                nearest.as_ref().unwrap().1.normal2.y,
                nearest,
            );
            // beyond this point, code is only run if we do hit a collision
            let mut remaining_time = delta_seconds;
            let mut iterations_left = 5;
            while let Some((_, toi)) = nearest {
                transform.prepend_translation(xy_with_zero_z(toi.toi * direction.as_ref()));
                remaining_time -= toi.toi / velocity.magnitude();
                iterations_left -= 1;
                // kill velocity towards our destination
                *velocity -= velocity.dot(toi.normal1.as_ref()) * toi.normal1.as_ref();
                direction = Unit::new_normalize(*velocity); // note: will be NaN if velocity == 0.0.
                isometry = transform.to_2d_isometry();
                maximum_distance = velocity.magnitude() * delta_seconds;
                if remaining_time <= 0.0 || iterations_left == 0 || maximum_distance == 0.0 {
                    break;
                }

                let interferences =
                    ncollide_world.interferences_with_aabb(&aabb, &presence.collision_groups);
                nearest = interferences
                    .filter_map(|(handle, x)| {
                        ncollide2d::query::time_of_impact(
                            &isometry,
                            &direction,
                            shape,
                            x.position(),
                            &nalgebra::zero(),
                            x.shape().as_ref(),
                            std::f32::MAX,
                            0.0,
                        )
                        .map(|toi| (handle, toi))
                    })
                    .min_by(|(_, toi1), (_, toi2)| toi1.toi.partial_cmp(&toi2.toi).unwrap());
                all_clear = nearest.is_none();
            }
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
