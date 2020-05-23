use amethyst::{
    core::{timing::Time, SystemDesc, Transform},
    derive::SystemDesc,
    ecs::prelude::*,
    input::{InputHandler, StringBindings},
};

use super::Ncollide2dWorld;
use crate::{
    collisions::{
        components::{HasGravity, Ncollide2dHandle},
        resolution_utils::on_floor,
    },
    world::*,
};

#[derive(SystemDesc)]
pub struct GravitySystem;

impl<'s> System<'s> for GravitySystem {
    type SystemData = (
        WriteStorage<'s, Velocity>,
        ReadStorage<'s, HasGravity>,
        ReadStorage<'s, Ncollide2dHandle>,
        Read<'s, Ncollide2dWorld>,
        Read<'s, Time>,
        ReadExpect<'s, ConstantsConfig>,
    );
    fn run(
        &mut self,
        (mut velocities, gravities, handles, ncollide_world, time, constants): Self::SystemData,
    ) {
        let ncollide_world = &ncollide_world.world;
        for (velocity, _, handle) in (&mut velocities, &gravities, &handles).join() {
            if !on_floor(ncollide_world, handle.0) {
                velocity.intended.y -= constants.gravity_accel * time.delta_seconds();
            }
        }
    }
}
