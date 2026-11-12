//! On-screen text: run progress, a controls reminder, and the finish cue.

use bevy::prelude::*;

use crate::run::Run;
use crate::visuals::palette;

#[derive(Component)]
struct Progress;

#[derive(Component)]
struct FinishCue;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hud)
            .add_systems(Update, show_run.run_if(resource_changed::<Run>));
    }
}

fn text(content: impl Into<String>, size: f32) -> impl Bundle {
    (
        Text::new(content),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(palette::TEXT),
    )
}

fn panel(node: Node) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
            ..node
        },
        BackgroundColor(palette::PANEL),
    )
}

fn spawn_hud(mut commands: Commands) {
    commands.spawn((
        Name::new("Progress"),
        Progress,
        panel(Node {
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        text("", 20.0),
    ));
    commands.spawn((
        Name::new("Controls reminder"),
        panel(Node {
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        text(
            "A/D or Left/Right: run    Space/W/Up: jump (hold for higher)    R: restart    Esc: quit",
            15.0,
        ),
    ));
    commands.spawn((
        Name::new("Finish cue"),
        FinishCue,
        panel(Node {
            top: Val::Percent(35.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        }),
        text("", 32.0),
        TextLayout::justify(Justify::Center),
        Visibility::Hidden,
    ));
}

fn show_run(
    run: Res<Run>,
    mut progress: Query<&mut Text, (With<Progress>, Without<FinishCue>)>,
    mut finish: Query<(&mut Text, &mut Visibility), With<FinishCue>>,
) {
    let summary = format!(
        "Coins {}/{}    Time {:.1} s",
        run.coins, run.total_coins, run.elapsed
    );
    for mut text in &mut progress {
        text.0.clone_from(&summary);
    }
    for (mut text, mut visibility) in &mut finish {
        if run.finished {
            text.0 = format!("Goal!  {summary}\nPress R to play again");
            *visibility = Visibility::Inherited;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}
