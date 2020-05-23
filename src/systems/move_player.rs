use amethyst::{
    core::{timing::Time, Transform},
    derive::SystemDesc,
    ecs::prelude::*,
    input::{InputHandler, StringBindings},
};
use log::debug;

use super::Ncollide2dWorld;
use crate::{
    collisions::{components::Ncollide2dHandle, resolution_utils::on_floor},
    world::*,
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
