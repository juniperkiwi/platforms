use amethyst::{
    core::transform::Transform, core::SystemDesc, derive::SystemDesc, ecs::prelude::*, ui::UiText,
};

use crate::pong::{Ball, ScoreText, Scoreboard, ARENA_WIDTH};

#[derive(SystemDesc)]
pub struct WinnerSystem;

impl<'s> System<'s> for WinnerSystem {
    type SystemData = (
        WriteStorage<'s, Ball>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, UiText>,
        Write<'s, Scoreboard>,
        ReadExpect<'s, ScoreText>,
    );

    fn run(
        &mut self,
        (mut balls, mut locals, mut ui_texts, mut scoreboard, score_text): Self::SystemData,
    ) {
        for (ball, transform) in (&mut balls, &mut locals).join() {
            let ball_x = transform.translation().x;

            let on_edge = if ball_x <= ball.radius {
                scoreboard.score_right += 1;
                if let Some(text) = ui_texts.get_mut(score_text.p2_score) {
                    text.text = scoreboard.score_right.to_string();
                }

                true
            } else if ball_x >= ARENA_WIDTH - ball.radius {
                scoreboard.score_left += 1;
                if let Some(text) = ui_texts.get_mut(score_text.p1_score) {
                    text.text = scoreboard.score_left.to_string();
                }
                true
            } else {
                false
            };

            if on_edge {
                ball.velocity[0] *= -1.0;
                transform.set_translation_x(ARENA_WIDTH / 2.0);
            }
        }
    }
}
