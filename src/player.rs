//! The player body: kinematic movement with tile collision on the fixed timestep.
//!
//! Nothing here reads devices or draws. Input arrives through [`Controls`], and
//! the result is stored in [`Position`], so the movement runs headless in tests.
//! See ADR 0001 for why this is hand-written instead of a physics engine.

use bevy::prelude::*;

use crate::level::Level;

/// Marks the player entity.
#[derive(Component, Debug)]
pub struct Player;

/// The world position of an entity's simulated anchor. For the player this is
/// the centre of the feet; for coins it is the coin's centre.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct Position(pub Vec2);

/// The position before the latest fixed step, used to interpolate rendering.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct PreviousPosition(pub Vec2);

/// Velocity in world units per second.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Velocity(pub Vec2);

/// Ground contact and the forgiving timers that make jumps feel responsive.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Grounding {
    pub grounded: bool,
    /// Time left to jump after walking off a ledge ("coyote time").
    pub coyote: f32,
    /// Time left for an early jump press to fire on landing.
    pub jump_buffer: f32,
}

/// Player intent for the fixed-step systems, written by the input layer.
///
/// `jump_pressed` and `restart` are latched until a fixed step consumes them,
/// so a press is never lost on a frame without a fixed step.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct Controls {
    /// Horizontal intent in `-1.0..=1.0`.
    pub move_x: f32,
    pub jump_held: bool,
    pub jump_pressed: bool,
    pub restart: bool,
}

/// Movement feel. These are play-test values: judge changes in the game.
#[derive(Resource, Clone, Copy, Debug)]
pub struct PlayerTuning {
    /// Collision box width and height.
    pub size: Vec2,
    pub run_speed: f32,
    pub ground_accel: f32,
    pub air_accel: f32,
    pub gravity: f32,
    pub max_fall_speed: f32,
    pub jump_speed: f32,
    /// Upward speed is capped to this fraction of `jump_speed` once jump is released.
    pub jump_cut: f32,
    pub coyote_time: f32,
    pub jump_buffer_time: f32,
}

impl Default for PlayerTuning {
    fn default() -> Self {
        Self {
            size: Vec2::new(24.0, 40.0),
            run_speed: 300.0,
            ground_accel: 2400.0,
            air_accel: 1500.0,
            gravity: 2000.0,
            max_fall_speed: 900.0,
            jump_speed: 680.0,
            jump_cut: 0.45,
            coyote_time: 0.1,
            jump_buffer_time: 0.12,
        }
    }
}

/// The player's bundle at a feet position, at rest.
pub fn player_bundle(feet: Vec2) -> impl Bundle {
    (
        Name::new("Player"),
        Player,
        Position(feet),
        PreviousPosition(feet),
        Velocity::default(),
        Grounding::default(),
    )
}

/// Places the player back at `feet`, at rest.
pub fn respawn(
    feet: Vec2,
    position: &mut Position,
    previous: &mut PreviousPosition,
    velocity: &mut Velocity,
    grounding: &mut Grounding,
) {
    position.0 = feet;
    previous.0 = feet;
    *velocity = Velocity::default();
    *grounding = Grounding::default();
}

/// Advances the player one fixed step: steer, jump, fall, then collide.
pub fn step_player(
    time: Res<Time>,
    tuning: Res<PlayerTuning>,
    level: Res<Level>,
    mut controls: ResMut<Controls>,
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
    let dt = time.delta_secs();
    let jump_pressed = std::mem::take(&mut controls.jump_pressed);
    for (mut position, mut previous, mut velocity, mut grounding) in &mut players {
        previous.0 = position.0;
        let g = &mut *grounding;
        let v = &mut velocity.0;

        g.coyote = if g.grounded {
            tuning.coyote_time
        } else {
            (g.coyote - dt).max(0.0)
        };
        g.jump_buffer = if jump_pressed {
            tuning.jump_buffer_time
        } else {
            (g.jump_buffer - dt).max(0.0)
        };

        let target = controls.move_x.clamp(-1.0, 1.0) * tuning.run_speed;
        let accel = if g.grounded {
            tuning.ground_accel
        } else {
            tuning.air_accel
        };
        v.x = move_towards(v.x, target, accel * dt);

        if g.jump_buffer > 0.0 && g.coyote > 0.0 {
            v.y = tuning.jump_speed;
            g.jump_buffer = 0.0;
            g.coyote = 0.0;
        }
        if !controls.jump_held && v.y > 0.0 {
            v.y = v.y.min(tuning.jump_speed * tuning.jump_cut);
        }
        v.y = (v.y - tuning.gravity * dt).max(-tuning.max_fall_speed);

        let (feet, contacts) = move_and_collide(position.0, tuning.size, *v * dt, &level.solids);
        position.0 = feet;
        if contacts.wall {
            v.x = 0.0;
        }
        if contacts.ground || contacts.ceiling {
            v.y = 0.0;
        }
        g.grounded = contacts.ground;
    }
}

fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    current + (target - current).clamp(-max_delta, max_delta)
}

/// The collision box of a body whose feet are at `feet`.
pub fn body_rect(feet: Vec2, size: Vec2) -> Rect {
    Rect::from_corners(
        feet - Vec2::new(0.5 * size.x, 0.0),
        feet + Vec2::new(0.5 * size.x, size.y),
    )
}

/// Which sides were blocked during a move.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Contacts {
    pub wall: bool,
    pub ground: bool,
    pub ceiling: bool,
}

/// Moves a box by `delta`, one axis at a time, stopping at the first solids.
///
/// Boxes that only touch are not overlapping, so a body resting on the ground
/// slides freely. Steps must stay smaller than a tile to avoid tunnelling; the
/// tuning's speeds at 64 Hz move at most a few units per step.
pub fn move_and_collide(feet: Vec2, size: Vec2, delta: Vec2, solids: &[Rect]) -> (Vec2, Contacts) {
    let mut feet = feet;
    let mut contacts = Contacts::default();

    feet.x += delta.x;
    let body = body_rect(feet, size);
    for solid in solids.iter().filter(|solid| overlaps(body, **solid)) {
        contacts.wall = true;
        if delta.x > 0.0 {
            feet.x = feet.x.min(solid.min.x - 0.5 * size.x);
        } else {
            feet.x = feet.x.max(solid.max.x + 0.5 * size.x);
        }
    }

    feet.y += delta.y;
    let body = body_rect(feet, size);
    for solid in solids.iter().filter(|solid| overlaps(body, **solid)) {
        if delta.y > 0.0 {
            contacts.ceiling = true;
            feet.y = feet.y.min(solid.min.y - size.y);
        } else {
            contacts.ground = true;
            feet.y = feet.y.max(solid.max.y);
        }
    }
    (feet, contacts)
}

fn overlaps(a: Rect, b: Rect) -> bool {
    a.min.x < b.max.x && a.max.x > b.min.x && a.min.y < b.max.y && a.max.y > b.min.y
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIZE: Vec2 = Vec2::new(20.0, 40.0);
    const FLOOR: Rect = Rect {
        min: Vec2::new(-100.0, -32.0),
        max: Vec2::new(100.0, 0.0),
    };

    #[test]
    fn lands_on_the_floor_and_slides_along_it() {
        let (feet, contacts) =
            move_and_collide(Vec2::new(0.0, 3.0), SIZE, Vec2::new(5.0, -10.0), &[FLOOR]);
        assert_eq!(feet, Vec2::new(5.0, 0.0));
        assert!(contacts.ground && !contacts.wall);
    }

    #[test]
    fn stops_at_walls_and_ceilings() {
        let wall = Rect::new(30.0, 0.0, 62.0, 100.0);
        let (feet, contacts) = move_and_collide(Vec2::ZERO, SIZE, Vec2::new(25.0, 0.0), &[wall]);
        assert_eq!(feet.x, 20.0);
        assert!(contacts.wall);

        let ceiling = Rect::new(-50.0, 50.0, 50.0, 60.0);
        let (feet, contacts) = move_and_collide(Vec2::ZERO, SIZE, Vec2::new(0.0, 15.0), &[ceiling]);
        assert_eq!(feet.y, 10.0);
        assert!(contacts.ceiling);
    }
}
