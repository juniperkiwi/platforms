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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PlayerMovementConstants {
    jump: f32,
    horizontal_acceleration: f32,
    horizontal_decrease_multiplier: f32,
    horizontal_decrease_secs_per_decrease: f32,
    air_boost_increase_min_velocity: f32,
    air_boost_increase_per_sec: f32,
    air_boost_decrease_per_sec: f32,
    air_boost_max: f32,
}

#[derive(SystemDesc)]
pub struct MovePlayerSystem;

impl<'s> System<'s> for MovePlayerSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Velocity>,
        WriteStorage<'s, Player>,
        ReadStorage<'s, Ncollide2dHandle>,
        Read<'s, InputHandler<StringBindings>>,
        Read<'s, Ncollide2dWorld>,
        Read<'s, Time>,
        ReadExpect<'s, ConstantsConfig>,
    );
    fn run(
        &mut self,
        (
            mut transforms,
            mut velocities,
            mut players,
            handles,
            input,
            ncollide_world,
            time,
            constants,
        ): Self::SystemData,
    ) {
        let constants = &constants.player;
        let ncollide_world = &ncollide_world.world;
        let delta_t = time.delta_seconds();
        for (transform, velocity, player, handle) in
            (&mut transforms, &mut velocities, &mut players, &handles).join()
        {
            let lr = input.axis_value("left_right");
            let jump = input.action_is_down("jump");
            let on_floor = on_floor(ncollide_world, handle.0);
            if let Some(lr) = lr {
                velocity.intended.x += lr as f32
                    * constants.horizontal_acceleration
                    * delta_t
                    * (1.0 + player.air_boost);
                velocity.intended.x = velocity.intended.x
                    * (1.0 - delta_t / constants.horizontal_decrease_secs_per_decrease)
                    + (velocity.intended.x * constants.horizontal_decrease_multiplier)
                        * (delta_t / constants.horizontal_decrease_secs_per_decrease);
            }
            // jumping
            if jump.unwrap_or(false) {
                if on_floor {
                    debug!("jumping from floor!");
                    velocity.intended.y += constants.jump;
                } else {
                    debug!("jumping but not on floor");
                }
            }
            // air boost
            if on_floor {
                player.air_boost =
                    0f32.max(player.air_boost - constants.air_boost_decrease_per_sec * delta_t);
            } else if velocity.intended.magnitude() >= constants.air_boost_increase_min_velocity {
                player.air_boost = constants
                    .air_boost_max
                    .min(player.air_boost + constants.air_boost_increase_per_sec * delta_t);
            }
        }
    }
}
