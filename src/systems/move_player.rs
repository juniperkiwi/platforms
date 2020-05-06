use amethyst::{
    core::{timing::Time, SystemDesc, Transform},
    derive::SystemDesc,
    ecs::prelude::*,
    input::{InputHandler, StringBindings},
};

use crate::collisions::*;
use crate::game::{Paddle, Side, ARENA_HEIGHT, PADDLE_HEIGHT, PADDLE_VELOCITY};
use crate::world::*;
use nalgebra::{Vector2, Vector3};

#[derive(SystemDesc)]
pub struct MovePlayerSystem;

impl<'s> System<'s> for MovePlayerSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Ncollide2dHandle>,
        ReadStorage<'s, Player>,
        Read<'s, InputHandler<StringBindings>>,
        Read<'s, Ncollide2dWorld>,
        Read<'s, Time>,
    );
    fn run(
        &mut self,
        (mut transforms, handles, players, input, ncollide_world, time): Self::SystemData,
    ) {
        let ncollide_world = &ncollide_world.world;
        for (transform, handle, _) in (&mut transforms, &handles, &players).join() {
            let lr = input.axis_value("left_right");
            let jump = input.action_is_down("jump");
            if let Some(lr) = lr {
                let scaled_amount = time.delta_seconds() * lr as f32 * PADDLE_VELOCITY;
                let movement_vector = Vector2::new(scaled_amount, 0.0);
                let wall = ncollide_world
                    .contacts_with(handle.0, true)
                    .into_iter()
                    .flat_map(|v| v)
                    .filter_map(|(_h1, _h2, _algo, manifold)| {
                        manifold.contacts().find(|c| {
                            c.contact.normal.as_ref().dot(&movement_vector) != 0.0
                                && c.contact.world1.coords.x.signum() == -scaled_amount.signum()
                        })
                    })
                    .next();
                if scaled_amount != 0.0 {
                    eprintln!(
                        "contacts: {:#?}",
                        ncollide_world
                            .contacts_with(handle.0, true)
                            .into_iter()
                            .flat_map(|v| v)
                            .map(|(h1, h2, _algo, manifold)| (
                                ncollide_world.objects.get(h1).unwrap().position(),
                                ncollide_world.objects.get(h2).unwrap().position(),
                                manifold
                            ))
                            .collect::<Vec<_>>()
                    );
                    eprintln!("contact at {:?}", wall);
                }
                if wall.is_none() {
                    transform.prepend_translation(Vector3::new(scaled_amount, 0.0, 0.0));
                }
            }
        }

        // eprintln!(
        //     "{:#?}",
        //     (&transforms).join()
        //         .into_iter()
        //         .map(|v| format!("{:?}", (v.translation())))
        //         .collect::<Vec<_>>()
        // );
    }
}
