//! The headless game: level, player movement, and run rules on the fixed timestep.
//!
//! This plugin adds no rendering, windowing, or device input, so tests can run
//! it with `MinimalPlugins`. Presentation plugins read its components and
//! resources and write only [`Controls`].

use bevy::prelude::*;

use crate::level::Level;
use crate::player::{self, Controls, PlayerTuning};
use crate::run::{self, Run};

/// The fixed-step game systems, in order. Presentation code that depends on
/// their results can order itself after this set.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GameSystems;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Level>()
            .init_resource::<PlayerTuning>()
            .init_resource::<Controls>()
            .init_resource::<Run>()
            .add_systems(Startup, (spawn_player, run::start_run))
            .add_systems(
                FixedUpdate,
                (
                    run::restart_run,
                    player::step_player,
                    run::collect_coins,
                    run::reach_goal,
                    run::fall_out,
                    run::tick_clock,
                )
                    .chain()
                    .in_set(GameSystems),
            );
    }
}

fn spawn_player(mut commands: Commands, level: Res<Level>) {
    commands.spawn(player::player_bundle(level.start));
}
