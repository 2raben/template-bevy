//! The level: a tile map parsed into solid rectangles, a start, a goal, and coins.
//!
//! The map is plain text so a level can be edited without tooling. Parsing is pure
//! and has no Bevy systems, so it is tested directly.

use bevy::prelude::*;

/// The side of one map tile, in world units.
pub const TILE: f32 = 32.0;

/// The starter level. Each row is one tile high; the first row is the top.
///
/// - `#` solid ground or wall
/// - `P` player start (exactly one)
/// - `G` goal flag (exactly one)
/// - `o` coin
/// - `.` or space: empty
pub const STARTER_MAP: &[&str] = &[
    "............................................................",
    "............................................................",
    "............................................................",
    "............................................................",
    "............................oooo...............oo...........",
    "..........ooo...............####..##........................",
    "....................##............##......................G.",
    "..P...............####............##...o.o.o.............###",
    "##########...#########...#####################....##########",
    "##########...#########...#####################....##########",
];

/// A parsed level, in world units with the origin at the map's bottom-left.
#[derive(Resource, Clone, Debug)]
pub struct Level {
    /// Solid tiles merged into horizontal runs.
    pub solids: Vec<Rect>,
    /// Where the player's feet start: the centre-bottom of the `P` tile.
    pub start: Vec2,
    /// The goal tile; touching it finishes the run.
    pub goal: Rect,
    /// Centres of the coins.
    pub coins: Vec<Vec2>,
    /// The area covered by the map.
    pub bounds: Rect,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LevelError {
    Empty,
    UnknownTile {
        row: usize,
        column: usize,
        tile: char,
    },
    MissingStart,
    MissingGoal,
    DuplicateStart,
    DuplicateGoal,
}

impl Level {
    pub fn parse(rows: &[&str]) -> Result<Self, LevelError> {
        let height = rows.len();
        let width = rows
            .iter()
            .map(|row| row.chars().count())
            .max()
            .unwrap_or(0);
        if height == 0 || width == 0 {
            return Err(LevelError::Empty);
        }
        let tile_rect = |row: usize, column: usize| {
            let min = Vec2::new(column as f32, (height - 1 - row) as f32) * TILE;
            Rect::from_corners(min, min + Vec2::splat(TILE))
        };

        let mut solids = Vec::new();
        let mut start = None;
        let mut goal = None;
        let mut coins = Vec::new();
        for (row, line) in rows.iter().enumerate() {
            let mut run: Option<Rect> = None;
            for (column, tile) in line.chars().enumerate() {
                let rect = tile_rect(row, column);
                if tile == '#' {
                    run = Some(run.map_or(rect, |run| run.union(rect)));
                    continue;
                }
                solids.extend(run.take());
                match tile {
                    '.' | ' ' => {}
                    'o' => coins.push(rect.center()),
                    'P' if start.is_some() => return Err(LevelError::DuplicateStart),
                    'P' => start = Some(Vec2::new(rect.center().x, rect.min.y)),
                    'G' if goal.is_some() => return Err(LevelError::DuplicateGoal),
                    'G' => goal = Some(rect),
                    tile => return Err(LevelError::UnknownTile { row, column, tile }),
                }
            }
            solids.extend(run);
        }

        Ok(Self {
            solids,
            start: start.ok_or(LevelError::MissingStart)?,
            goal: goal.ok_or(LevelError::MissingGoal)?,
            coins,
            bounds: Rect::new(0.0, 0.0, width as f32 * TILE, height as f32 * TILE),
        })
    }

    pub fn starter() -> Self {
        Self::parse(STARTER_MAP).expect("the starter map is valid")
    }

    /// Below this height the player has fallen out of the level.
    pub fn fall_limit(&self) -> f32 {
        self.bounds.min.y - 4.0 * TILE
    }
}

impl Default for Level {
    fn default() -> Self {
        Self::starter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starter_map_parses() {
        let level = Level::starter();
        assert!(!level.solids.is_empty());
        assert!(!level.coins.is_empty());
        assert!(level.bounds.contains(level.start));
        assert!(level.bounds.contains(level.goal.center()));
    }

    #[test]
    fn merges_solid_runs_and_flips_rows() {
        let level = Level::parse(&["P..G", "##.#"]).unwrap();
        assert_eq!(
            level.solids,
            vec![
                Rect::new(0.0, 0.0, 2.0 * TILE, TILE),
                Rect::new(3.0 * TILE, 0.0, 4.0 * TILE, TILE),
            ]
        );
        assert_eq!(level.start, Vec2::new(0.5 * TILE, TILE));
        assert_eq!(
            level.goal,
            Rect::new(3.0 * TILE, TILE, 4.0 * TILE, 2.0 * TILE)
        );
    }

    #[test]
    fn rejects_invalid_maps() {
        assert_eq!(Level::parse(&[]).unwrap_err(), LevelError::Empty);
        assert_eq!(
            Level::parse(&["..G"]).unwrap_err(),
            LevelError::MissingStart
        );
        assert_eq!(Level::parse(&["P.."]).unwrap_err(), LevelError::MissingGoal);
        assert_eq!(
            Level::parse(&["PPG"]).unwrap_err(),
            LevelError::DuplicateStart
        );
        assert_eq!(
            Level::parse(&["P?G"]).unwrap_err(),
            LevelError::UnknownTile {
                row: 0,
                column: 1,
                tile: '?'
            }
        );
    }
}
