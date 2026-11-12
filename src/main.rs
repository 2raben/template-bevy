//! A small 2D platformer starter: run, jump, collect coins, and reach the flag.

mod camera;
mod hud;
mod input;
mod level;
mod player;
mod run;
mod simulation;
mod visuals;

#[cfg(test)]
mod tests;

use bevy::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Template".into(),
                resolution: (960, 540).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            simulation::SimulationPlugin,
            input::InputPlugin,
            visuals::VisualsPlugin,
            camera::CameraPlugin,
            hud::HudPlugin,
        ))
        .run()
}
