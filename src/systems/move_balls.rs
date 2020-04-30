use amethyst::{
    core::timing::Time, core::SystemDesc, core::Transform, derive::SystemDesc, ecs::prelude::*,
};

use crate::pong::Ball;

#[derive(SystemDesc)]
pub struct MoveBallsSystem;

impl<'s> System<'s> for MoveBallsSystem {
    type SystemData = (
        ReadStorage<'s, Ball>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
    );

    fn run(&mut self, (balls, mut transforms, time): Self::SystemData) {
        for (ball, transform) in (&balls, &mut transforms).join() {
            transform.prepend_translation_x(ball.velocity[0] * time.delta_seconds());
            transform.prepend_translation_y(ball.velocity[1] * time.delta_seconds());
        }
    }
}
