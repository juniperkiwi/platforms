use amethyst::{
    core::{Transform, SystemDesc}, derive::SystemDesc,
    ecs::prelude::*,
};

use crate::pong::{Ball,Side,Paddle,ARENA_HEIGHT};

#[derive(SystemDesc)]
pub struct BounceSystem;

impl<'s> System<'s> for BounceSystem {
    type SystemData = (
        WriteStorage<'s, Ball>,
        WriteStorage<'s, Paddle>,
        ReadStorage<'s, Transform>,
    );

    fn run(&mut self, (mut balls, paddles, transforms): Self::SystemData) {
        for (ball, transform) in (&mut balls, &transforms).join() {
            let ball_x = transform.translation().x;
            let ball_y = transform.translation().y;

            if (ball_y <= ball.radius && ball.velocity[1] < 0.0)
                || (ball_y >= ARENA_HEIGHT - ball.radius && ball.velocity[1] > 0.0)
            {
                dbg!(("bounce 1", ball_x, ball_y, ball.velocity));
                ball.velocity[1] *= -1.0;
            }

            for (paddle, paddle_transform) in (&paddles, &transforms).join() {
                let paddle_x = paddle_transform.translation().x - (paddle.width * 0.5);
                let paddle_y = paddle_transform.translation().y - (paddle.height * 0.5);

                if point_in_rect(
                    ball_x,
                    ball_y,
                    paddle_x - ball.radius,
                    paddle_y - ball.radius,
                    paddle_x + paddle.width + ball.radius,
                    paddle_y + paddle.height + ball.radius,
                ) {
                    if (paddle.side == Side::Left && ball.velocity[0] < 0.0)
                        || (paddle.side == Side::Right && ball.velocity[0] > 0.0)
                    {
                        dbg!(("bounce 2", ball_x, ball_y, ball.velocity, paddle_x, paddle_y));
                        ball.velocity[0] *= -1.0;
                    }
                }
            }
        }
    }
}

fn point_in_rect(x: f32, y: f32, left: f32, bottom: f32, right: f32, top: f32) -> bool {
    x >= left && x <= right && y >= bottom && y <= top
}
