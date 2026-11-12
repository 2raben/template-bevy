//! Headless simulation tests: the real game systems run at the fixed timestep
//! with inputs written directly to [`Controls`]. No window or GPU is needed.

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use crate::level::{Level, TILE};
use crate::player::{Controls, Grounding, Player, PlayerTuning, Position, Velocity};
use crate::run::{Coin, Run};
use crate::simulation::SimulationPlugin;

/// Bevy's default fixed timestep; each update advances exactly one step.
const STEP: f32 = 1.0 / 64.0;

/// A flat floor two tiles thick with the player on it and the goal far away.
const FLAT: &[&str] = &[
    "..............................",
    "..............................",
    "..............................",
    "..............................",
    "..............................",
    "..P..........................G",
    "##############################",
    "##############################",
];

struct Sim {
    app: App,
}

impl Sim {
    fn new(map: &[&str]) -> Self {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin))
            .insert_resource(Level::parse(map).expect("test map is valid"))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                STEP,
            )));
        app.finish();
        app.cleanup();
        let mut sim = Self { app };
        // The first update runs startup; later ones advance one fixed step each.
        sim.app.update();
        sim
    }

    fn steps(&mut self, count: usize) {
        for _ in 0..count {
            self.app.update();
        }
    }

    fn seconds(&mut self, seconds: f32) {
        self.steps((seconds / STEP).round() as usize);
    }

    fn controls(&mut self) -> Mut<'_, Controls> {
        self.app.world_mut().resource_mut::<Controls>()
    }

    fn press_jump(&mut self) {
        let mut controls = self.controls();
        controls.jump_pressed = true;
        controls.jump_held = true;
    }

    fn player<T: Component + Copy>(&mut self) -> T {
        let world = self.app.world_mut();
        *world
            .query_filtered::<&T, With<Player>>()
            .single(world)
            .expect("one player")
    }

    fn feet(&mut self) -> Vec2 {
        self.player::<Position>().0
    }

    fn grounded(&mut self) -> bool {
        self.player::<Grounding>().grounded
    }

    fn run(&self) -> Run {
        self.app.world().resource::<Run>().clone()
    }

    fn coins_left(&mut self) -> usize {
        let world = self.app.world_mut();
        world.query_filtered::<(), With<Coin>>().iter(world).count()
    }

    fn tuning(&self) -> PlayerTuning {
        *self.app.world().resource::<PlayerTuning>()
    }

    /// Steps until the player lands again, returning the highest feet height.
    fn apex_until_landing(&mut self) -> f32 {
        let mut apex = self.feet().y;
        self.steps(2);
        for _ in 0..256 {
            apex = apex.max(self.feet().y);
            if self.grounded() {
                return apex;
            }
            self.steps(1);
        }
        panic!("the player never landed");
    }
}

const FLOOR_TOP: f32 = 2.0 * TILE;

#[test]
fn player_starts_resting_on_the_floor() {
    let mut sim = Sim::new(FLAT);
    sim.seconds(0.5);
    assert!(sim.grounded());
    assert_eq!(sim.feet().y, FLOOR_TOP);
}

#[test]
fn running_reaches_and_keeps_run_speed() {
    let mut sim = Sim::new(FLAT);
    let start = sim.feet().x;
    sim.controls().move_x = 1.0;
    sim.seconds(1.0);
    let speed = sim.player::<Velocity>().0.x;
    assert!(
        (speed - sim.tuning().run_speed).abs() < 1e-3,
        "speed {speed}"
    );
    assert!(sim.feet().x - start > 0.8 * sim.tuning().run_speed);

    sim.controls().move_x = 0.0;
    sim.seconds(0.5);
    assert_eq!(
        sim.player::<Velocity>().0.x,
        0.0,
        "releasing stops the player"
    );
}

#[test]
fn held_jump_clears_three_tiles_and_a_tap_hops_lower() {
    let mut sim = Sim::new(FLAT);
    sim.press_jump();
    let full = sim.apex_until_landing() - FLOOR_TOP;
    assert!((3.0 * TILE..4.0 * TILE).contains(&full), "full jump {full}");

    sim.controls().jump_pressed = true;
    sim.steps(1);
    sim.controls().jump_held = false;
    let hop = sim.apex_until_landing() - FLOOR_TOP;
    assert!(hop < 0.5 * full, "tap {hop} vs held {full}");
}

/// The starter level's widest gap is four tiles; a running jump must clear it.
#[test]
fn running_jump_clears_a_four_tile_gap() {
    let mut sim = Sim::new(&[
        "........................",
        "........................",
        "........................",
        "........................",
        ".P....................G.",
        "##########....##########",
    ]);
    sim.controls().move_x = 1.0;
    let edge = 10.0 * TILE;
    // Jump from the last moment the body still overlaps the ledge.
    while sim.feet().x + 0.5 * sim.tuning().size.x + 4.0 < edge {
        sim.steps(1);
    }
    sim.press_jump();
    sim.seconds(1.0);
    assert_eq!(sim.feet().y, TILE, "landed beyond the gap");
    assert_eq!(sim.run().falls, 0);
}

#[test]
fn no_jumping_again_in_midair() {
    let mut sim = Sim::new(FLAT);
    sim.press_jump();
    sim.seconds(0.4);
    assert!(!sim.grounded());
    let before = sim.player::<Velocity>().0.y;
    sim.press_jump();
    sim.steps(1);
    assert!(sim.player::<Velocity>().0.y < before);
}

#[test]
fn an_early_jump_press_fires_on_landing() {
    let mut sim = Sim::new(FLAT);
    sim.press_jump();
    // Fall until a few steps above the floor, then press while still airborne.
    while !(sim.player::<Velocity>().0.y < 0.0 && sim.feet().y < FLOOR_TOP + 20.0) {
        sim.steps(1);
    }
    assert!(!sim.grounded());
    sim.press_jump();
    sim.seconds(0.2);
    assert!(
        sim.feet().y > FLOOR_TOP + TILE,
        "the buffered press launched a new jump"
    );
}

#[test]
fn walls_stop_the_player() {
    let mut sim = Sim::new(&["........", "........", "....#...", ".P..#..G", "########"]);
    sim.controls().move_x = 1.0;
    sim.seconds(1.0);
    let half_width = 0.5 * sim.tuning().size.x;
    assert_eq!(sim.feet().x, 4.0 * TILE - half_width);
    assert!(sim.grounded());
}

#[test]
fn coins_are_collected_once() {
    let mut sim = Sim::new(&[".....", ".Po.G", "#####"]);
    assert_eq!(sim.run().total_coins, 1);
    sim.controls().move_x = 1.0;
    sim.seconds(0.4);
    sim.controls().move_x = 0.0;
    assert_eq!(sim.run().coins, 1);
    assert_eq!(sim.coins_left(), 0);
}

#[test]
fn touching_the_goal_finishes_and_stops_the_clock() {
    let mut sim = Sim::new(&[".....", ".P..G", "#####"]);
    sim.controls().move_x = 1.0;
    sim.seconds(1.0);
    let run = sim.run();
    assert!(run.finished);
    sim.seconds(0.5);
    assert_eq!(sim.run().elapsed, run.elapsed);
}

#[test]
fn falling_out_returns_to_the_start() {
    let mut sim = Sim::new(&["......", ".P...G", "##....", "##...."]);
    let start = sim.feet();
    sim.controls().move_x = 1.0;
    sim.seconds(0.5);
    sim.controls().move_x = 0.0;
    sim.seconds(2.0);
    assert_eq!(sim.run().falls, 1);
    assert_eq!(sim.feet(), start);
}

#[test]
fn restart_resets_the_player_coins_and_clock() {
    let mut sim = Sim::new(&[".....", ".Po.G", "#####"]);
    let start = sim.feet();
    sim.controls().move_x = 1.0;
    sim.seconds(0.4);
    sim.controls().move_x = 0.0;
    assert_eq!(sim.run().coins, 1);

    sim.controls().restart = true;
    sim.steps(2);
    let run = sim.run();
    assert_eq!((run.coins, run.falls, run.finished), (0, 0, false));
    assert!(run.elapsed < 2.5 * STEP);
    assert_eq!(sim.coins_left(), 1);
    assert_eq!(sim.feet(), start);
}
