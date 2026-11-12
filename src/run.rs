//! The rules of one run through the level: coins, the goal, falling out, and
//! restarting. Like the player movement, these run headless on the fixed timestep.

use bevy::prelude::*;

use crate::level::Level;
use crate::player::{
    Controls, Grounding, Player, PlayerTuning, Position, PreviousPosition, Velocity, body_rect,
    respawn,
};

/// Marks a collectable coin. Its [`Position`] is the coin's centre.
#[derive(Component, Debug)]
pub struct Coin;

/// Coins within this distance of the player's box are collected.
pub const COIN_RADIUS: f32 = 10.0;

/// Progress through the current run.
#[derive(Resource, Clone, Debug, Default)]
pub struct Run {
    pub coins: usize,
    pub total_coins: usize,
    /// Seconds since the run started; stops when the goal is reached.
    pub elapsed: f32,
    pub finished: bool,
    /// Times the player fell out of the level during this run.
    pub falls: u32,
}

pub fn coin_bundle(center: Vec2) -> impl Bundle {
    (Name::new("Coin"), Coin, Position(center))
}

/// Spawns the level's coins and resets the run.
pub fn start_run(mut commands: Commands, level: Res<Level>, mut run: ResMut<Run>) {
    for &center in &level.coins {
        commands.spawn(coin_bundle(center));
    }
    *run = Run {
        total_coins: level.coins.len(),
        ..default()
    };
}

/// Consumes a restart request: the player, coins, and run start over.
pub fn restart_run(
    mut commands: Commands,
    level: Res<Level>,
    mut controls: ResMut<Controls>,
    mut run: ResMut<Run>,
    coins: Query<Entity, With<Coin>>,
    mut players: Query<
        (
            &mut Position,
            &mut PreviousPosition,
            &mut Velocity,
            &mut Grounding,
        ),
        With<Player>,
    >,
) {
    if !std::mem::take(&mut controls.restart) {
        return;
    }
    for (mut position, mut previous, mut velocity, mut grounding) in &mut players {
        respawn(
            level.start,
            &mut position,
            &mut previous,
            &mut velocity,
            &mut grounding,
        );
    }
    for coin in &coins {
        commands.entity(coin).despawn();
    }
    for &center in &level.coins {
        commands.spawn(coin_bundle(center));
    }
    *run = Run {
        total_coins: level.coins.len(),
        ..default()
    };
}

pub fn collect_coins(
    mut commands: Commands,
    tuning: Res<PlayerTuning>,
    mut run: ResMut<Run>,
    players: Query<&Position, With<Player>>,
    coins: Query<(Entity, &Position), With<Coin>>,
) {
    for player in &players {
        let body = body_rect(player.0, tuning.size);
        for (coin, center) in &coins {
            let nearest = center.0.clamp(body.min, body.max);
            if nearest.distance(center.0) <= COIN_RADIUS {
                commands.entity(coin).despawn();
                run.coins += 1;
            }
        }
    }
}

pub fn reach_goal(
    level: Res<Level>,
    tuning: Res<PlayerTuning>,
    mut run: ResMut<Run>,
    players: Query<&Position, With<Player>>,
) {
    if run.finished {
        return;
    }
    let reached = players.iter().any(|player| {
        !body_rect(player.0, tuning.size)
            .intersect(level.goal)
            .is_empty()
    });
    if reached {
        run.finished = true;
    }
}

/// Returns the player to the start after falling below the level. Collected
/// coins and the clock carry on; only Restart resets the run.
pub fn fall_out(
    level: Res<Level>,
    mut run: ResMut<Run>,
    mut players: Query<
        (
            &mut Position,
            &mut PreviousPosition,
            &mut Velocity,
            &mut Grounding,
        ),
        With<Player>,
    >,
) {
    for (mut position, mut previous, mut velocity, mut grounding) in &mut players {
        if position.0.y < level.fall_limit() {
            respawn(
                level.start,
                &mut position,
                &mut previous,
                &mut velocity,
                &mut grounding,
            );
            run.falls += 1;
        }
    }
}

pub fn tick_clock(time: Res<Time>, mut run: ResMut<Run>) {
    if !run.finished {
        run.elapsed += time.delta_secs();
    }
}
