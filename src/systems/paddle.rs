use amethyst::{
    core::{timing::Time, Transform},
    derive::SystemDesc,
    ecs::prelude::*,
    input::{InputHandler, StringBindings},
};

use crate::game::{Paddle, Side, ARENA_HEIGHT, PADDLE_HEIGHT, PADDLE_VELOCITY};

#[derive(SystemDesc)]
pub struct PaddleSystem;

impl<'s> System<'s> for PaddleSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Paddle>,
        Read<'s, InputHandler<StringBindings>>,
        Read<'s, Time>,
    );
    fn run(&mut self, (mut transforms, paddles, input, time): Self::SystemData) {
        for (paddle, transform) in (&paddles, &mut transforms).join() {
            let movement = match paddle.side {
                Side::Left => input.axis_value("left_paddle"),
                Side::Right => input.axis_value("right_paddle"),
            };
            if let Some(mv_amount) = movement {
                let scaled_amount = time.delta_seconds() * PADDLE_VELOCITY * mv_amount as f32;
                let paddle_y = transform.translation().y;
                transform.set_translation_y(
                    (paddle_y + scaled_amount)
                        .min(ARENA_HEIGHT - PADDLE_HEIGHT * 0.5)
                        .max(PADDLE_HEIGHT * 0.5),
                );
            }
        }
    }
}
