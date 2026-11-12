//! A side-scrolling camera that follows the player and stays inside the level.

use bevy::camera::ScalingMode;
use bevy::prelude::*;

use crate::level::Level;
use crate::player::Player;

/// The minimum visible area, in world units.
pub const VIEW: Vec2 = Vec2::new(960.0, 540.0);

/// The view leads the player slightly to the right, where the level continues.
const LOOK_AHEAD: Vec2 = Vec2::new(120.0, 40.0);
const FOLLOW_RATE: f32 = 6.0;
/// Larger jumps, such as a respawn, snap instead of gliding.
const SNAP_DISTANCE: f32 = 600.0;

/// Camera movement in `PostUpdate`, after the player's transform is placed.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CameraSystems;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera).add_systems(
            PostUpdate,
            follow_player
                .in_set(CameraSystems)
                .before(TransformSystems::Propagate),
        );
    }
}

fn spawn_camera(mut commands: Commands, level: Res<Level>) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: VIEW.x,
                min_height: VIEW.y,
            },
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_translation(camera_target(&level, level.start, VIEW * 0.5).extend(0.0)),
    ));
}

/// Where the camera should centre for a player at `player`, keeping a view of
/// `half_view` inside the level where it fits and centring it where it does not.
pub fn camera_target(level: &Level, player: Vec2, half_view: Vec2) -> Vec2 {
    let wanted = player + LOOK_AHEAD;
    let bounds = level.bounds;
    let axis = |wanted: f32, min: f32, max: f32, half: f32| {
        if max - min <= 2.0 * half {
            0.5 * (min + max)
        } else {
            wanted.clamp(min + half, max - half)
        }
    };
    Vec2::new(
        axis(wanted.x, bounds.min.x, bounds.max.x, half_view.x),
        axis(wanted.y, bounds.min.y, bounds.max.y, half_view.y),
    )
}

fn follow_player(
    time: Res<Time>,
    level: Res<Level>,
    players: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    mut cameras: Query<(&mut Transform, &Projection), With<Camera2d>>,
) {
    let Ok(player) = players.single() else {
        return;
    };
    for (mut camera, projection) in &mut cameras {
        let half_view = match projection {
            Projection::Orthographic(ortho) => ortho.area.half_size(),
            _ => VIEW * 0.5,
        };
        let target = camera_target(&level, player.translation.truncate(), half_view);
        let current = camera.translation.truncate();
        let next = if current.distance(target) > SNAP_DISTANCE {
            target
        } else {
            let blend = 1.0 - (-FOLLOW_RATE * time.delta_secs()).exp();
            current.lerp(target, blend)
        };
        camera.translation = next.extend(camera.translation.z);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_view_inside_the_level() {
        let level = Level::starter();
        let half = VIEW * 0.5;
        let at_start = camera_target(&level, level.start, half);
        assert_eq!(at_start.x, level.bounds.min.x + half.x);
        // The starter level is shorter than the view, so it is centred vertically.
        assert_eq!(at_start.y, level.bounds.center().y);
        let at_goal = camera_target(&level, level.goal.center(), half);
        assert_eq!(at_goal.x, level.bounds.max.x - half.x);
    }
}
