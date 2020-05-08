use amethyst::{
    core::{timing::Time, SystemDesc, Transform},
    derive::SystemDesc,
    ecs::prelude::*,
    input::{InputHandler, StringBindings},
};

use crate::{
    collisions::*,
    game::{Paddle, Side, ARENA_HEIGHT, PADDLE_HEIGHT, PADDLE_VELOCITY},
    world::*,
};
use nalgebra::{Vector2, Vector3};
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
                let scaled_amount = time.delta_seconds() * lr as f32 * PADDLE_VELOCITY;
                let wall = ncollide_world
                    .contacts_with(handle.0, true)
                    .into_iter()
                    .flat_map(|v| v)
                    .find(|(_h1, _h2, _algo, manifold)| {
                        let c = manifold.deepest_contact();
                        c.map(|c| {
                            c.contact.normal.as_ref().x.partial_cmp(&0.0)
                                == (lr as f32).partial_cmp(&0.0)
                        })
                        .unwrap_or(false)
                    });
                if scaled_amount != 0.0 || jump.unwrap_or(false) {
                    eprintln!(
                        "contacts: [{}]",
                        ncollide_world
                            .contacts_with(handle.0, true)
                            .into_iter()
                            .flat_map(|v| v)
                            .map(format_contact!(ncollide_world))
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    if let Some(wall) = wall {
                        eprintln!("wall contact at {}", format_contact!(ncollide_world)(wall));
                    }
                }
                if wall.is_none() {
                    transform.prepend_translation(Vector3::new(scaled_amount, 0.0, 0.0));
                }
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
            let ceiling = ncollide_world
                .contacts_with(handle.0, true)
                .into_iter()
                .flat_map(|v| v)
                .find(|(_h1, _h2, _algo, manifold)| {
                    let c = manifold.deepest_contact();
                    c.map(|c| c.contact.normal.as_ref().y > 0.0)
                        .unwrap_or(false)
                });
            if floor.is_none() {
                velocity.intended.y -= GRAVITY_ACCEL;
            } else if velocity.intended.y < 0.0 {
                velocity.intended.y = 0.0;
                eprintln!(
                    "floor contact at {}",
                    format_contact!(ncollide_world)(floor.unwrap())
                );
            }
            if ceiling.is_some() && velocity.intended.y > 0.0 {
                velocity.intended.y = -velocity.intended.y;
                eprintln!(
                    "ceiling contact at {}",
                    format_contact!(ncollide_world)(ceiling.unwrap())
                );
            }
            transform.prepend_translation(Vector3::new(
                0.0,
                velocity.intended.y * time.delta_seconds(),
                0.0,
            ));
        }
    }
}

#[derive(SystemDesc)]
pub struct ApplyIntendedVelocityUpdatePredictedPositions;

impl<'s> System<'s> for ApplyIntendedVelocityUpdatePredictedPositions {
    type SystemData = (
        ReadStorage<'s, Transform>,
        WriteStorage<'s, Velocity>,
        ReadStorage<'s, Ncollide2dHandle>,
        Write<'s, Ncollide2dWorld>,
        Read<'s, Time>,
    );
    fn run(&mut self, (transforms, velocities, handles, world, time): Self::SystemData) {}
}
