//! Drawing: coloured sprites for the level, goal, coins, and player, and
//! interpolation of simulated positions into transforms.
//!
//! Simulation entities carry no sprites. This plugin decorates them when they
//! appear, so the headless simulation never needs assets or a renderer.

use bevy::prelude::*;

use crate::level::{Level, TILE};
use crate::player::{Player, PlayerTuning, Position, PreviousPosition, Velocity};
use crate::run::Coin;

pub mod palette {
    use bevy::prelude::Color;

    pub const SKY: Color = Color::srgb(0.53, 0.75, 0.92);
    pub const GROUND: Color = Color::srgb(0.42, 0.29, 0.2);
    pub const GRASS: Color = Color::srgb(0.36, 0.7, 0.3);
    pub const PLAYER: Color = Color::srgb(0.93, 0.36, 0.3);
    pub const EYE: Color = Color::srgb(0.1, 0.1, 0.14);
    pub const COIN: Color = Color::srgb(0.99, 0.8, 0.2);
    pub const POLE: Color = Color::srgb(0.9, 0.9, 0.92);
    pub const FLAG: Color = Color::srgb(0.25, 0.55, 0.95);
    pub const PANEL: Color = Color::srgba(0.05, 0.08, 0.12, 0.8);
    pub const TEXT: Color = Color::srgb(0.95, 0.96, 0.98);
}

/// Draw order, back to front.
mod layer {
    pub const LEVEL: f32 = 0.0;
    pub const GOAL: f32 = 1.0;
    pub const COIN: f32 = 2.0;
    pub const PLAYER: f32 = 3.0;
}

const GRASS_DEPTH: f32 = 6.0;

pub struct VisualsPlugin;

impl Plugin for VisualsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(palette::SKY))
            .add_systems(Startup, draw_level)
            .add_systems(Update, (decorate_player, decorate_coins))
            .add_systems(
                PostUpdate,
                (interpolate_positions, face_movement)
                    .before(TransformSystems::Propagate)
                    .before(crate::camera::CameraSystems),
            );
    }
}

fn sprite(color: Color, size: Vec2, center: Vec2, z: f32) -> impl Bundle {
    (
        Sprite::from_color(color, size),
        Transform::from_translation(center.extend(z)),
    )
}

fn draw_level(mut commands: Commands, level: Res<Level>) {
    let solid_at = |point: Vec2| level.solids.iter().any(|solid| solid.contains(point));
    for solid in &level.solids {
        commands.spawn((
            Name::new("Ground"),
            sprite(palette::GROUND, solid.size(), solid.center(), layer::LEVEL),
        ));
        // Grass on each tile whose top is exposed to the sky.
        let columns = (solid.width() / TILE).round() as usize;
        for column in 0..columns {
            let x = solid.min.x + (column as f32 + 0.5) * TILE;
            if !solid_at(Vec2::new(x, solid.max.y + 0.5 * TILE)) {
                commands.spawn(sprite(
                    palette::GRASS,
                    Vec2::new(TILE, GRASS_DEPTH),
                    Vec2::new(x, solid.max.y - 0.5 * GRASS_DEPTH),
                    layer::LEVEL + 0.1,
                ));
            }
        }
    }

    let goal = level.goal;
    let pole = Vec2::new(4.0, goal.height() * 2.0);
    let pole_center = Vec2::new(goal.center().x, goal.min.y + 0.5 * pole.y);
    commands.spawn((
        Name::new("Goal pole"),
        sprite(palette::POLE, pole, pole_center, layer::GOAL),
    ));
    commands.spawn((
        Name::new("Goal flag"),
        sprite(
            palette::FLAG,
            Vec2::new(22.0, 14.0),
            Vec2::new(goal.center().x + 13.0, goal.min.y + pole.y - 9.0),
            layer::GOAL,
        ),
    ));
}

fn decorate_player(
    mut commands: Commands,
    tuning: Res<PlayerTuning>,
    players: Query<(Entity, &Position), Added<Player>>,
) {
    let size = tuning.size;
    for (player, position) in &players {
        commands
            .entity(player)
            .insert((
                Transform::from_translation(position.0.extend(layer::PLAYER)),
                Visibility::default(),
            ))
            .with_children(|body| {
                // Children are offset so the parent's origin stays at the feet.
                body.spawn(sprite(
                    palette::PLAYER,
                    size,
                    Vec2::new(0.0, 0.5 * size.y),
                    0.0,
                ));
                body.spawn(sprite(
                    palette::EYE,
                    Vec2::new(4.0, 8.0),
                    Vec2::new(0.22 * size.x, 0.72 * size.y),
                    0.1,
                ));
            });
    }
}

fn decorate_coins(mut commands: Commands, coins: Query<(Entity, &Position), Added<Coin>>) {
    for (coin, position) in &coins {
        commands.entity(coin).insert((
            Sprite::from_color(palette::COIN, Vec2::splat(14.0)),
            // A square turned on its corner reads as a gem-like coin.
            Transform::from_translation(position.0.extend(layer::COIN))
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
        ));
    }
}

/// Places moving bodies between their last two fixed-step positions, so motion
/// stays smooth when the display rate differs from the fixed rate.
fn interpolate_positions(
    fixed: Res<Time<Fixed>>,
    mut bodies: Query<(&Position, &PreviousPosition, &mut Transform)>,
) {
    let blend = fixed.overstep_fraction();
    for (position, previous, mut transform) in &mut bodies {
        let at = previous.0.lerp(position.0, blend);
        transform.translation = at.extend(transform.translation.z);
    }
}

fn face_movement(mut players: Query<(&Velocity, &mut Transform), With<Player>>) {
    for (velocity, mut transform) in &mut players {
        if velocity.0.x.abs() > 1.0 {
            transform.scale.x = velocity.0.x.signum();
        }
    }
}
