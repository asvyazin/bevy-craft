use bevy::prelude::*;
use bevy::ui::Val;

/// Marker component for the status UI root node
#[derive(Component)]
pub struct StatusUI;

/// Marker component for health bar
#[derive(Component)]
pub struct HealthBar;

/// Marker component for health bar fill
#[derive(Component)]
pub struct HealthBarFill;

/// Marker component for hunger bar
#[derive(Component)]
pub struct HungerBar;

/// Marker component for hunger bar fill
#[derive(Component)]
pub struct HungerBarFill;

/// Marker component for thirst bar
#[derive(Component)]
pub struct ThirstBar;

/// Marker component for thirst bar fill
#[derive(Component)]
pub struct ThirstBarFill;

/// Marker component for status labels
#[derive(Component)]
pub struct StatusLabel {
    pub label_type: StatusLabelType,
}

/// Type of status label
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StatusLabelType {
    Health,
    Hunger,
    Thirst,
}

/// System to spawn the status UI
pub fn spawn_status_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                left: Val::Px(20.0),
                width: Val::Px(200.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            StatusUI,
        ))
        .with_children(|parent| {
            spawn_health_bar(parent);
            spawn_hunger_bar(parent);
            spawn_thirst_bar(parent);
        });
}

fn spawn_health_bar(parent: &mut ChildBuilder) {
    parent
        .spawn((
            Node {
                width: Val::Px(200.0),
                height: Val::Px(30.0),
                ..default()
            },
            HealthBar,
        ))
        .with_children(|bar_parent| {
            bar_parent.spawn((
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(30.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.8, 0.2, 0.2, 0.9)),
                HealthBarFill,
            ));
        });

    parent.spawn((
        Text::new("Health: 100/100"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        StatusLabel {
            label_type: StatusLabelType::Health,
        },
    ));
}

fn spawn_hunger_bar(parent: &mut ChildBuilder) {
    parent
        .spawn((
            Node {
                width: Val::Px(200.0),
                height: Val::Px(30.0),
                ..default()
            },
            HungerBar,
        ))
        .with_children(|bar_parent| {
            bar_parent.spawn((
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(30.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.8, 0.6, 0.2, 0.9)),
                HungerBarFill,
            ));
        });

    parent.spawn((
        Text::new("Hunger: 100/100"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        StatusLabel {
            label_type: StatusLabelType::Hunger,
        },
    ));
}

fn spawn_thirst_bar(parent: &mut ChildBuilder) {
    parent
        .spawn((
            Node {
                width: Val::Px(200.0),
                height: Val::Px(30.0),
                ..default()
            },
            ThirstBar,
        ))
        .with_children(|bar_parent| {
            bar_parent.spawn((
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(30.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.2, 0.5, 0.9, 0.9)),
                ThirstBarFill,
            ));
        });

    parent.spawn((
        Text::new("Thirst: 100/100"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        StatusLabel {
            label_type: StatusLabelType::Thirst,
        },
    ));
}

/// System to update the status UI based on player stats
pub fn update_status_ui(
    player_query: Query<&crate::player::Player>,
    mut fill_queries: ParamSet<(
        Query<&mut Node, With<HealthBarFill>>,
        Query<&mut Node, With<HungerBarFill>>,
        Query<&mut Node, With<ThirstBarFill>>,
    )>,
    mut status_label_query: Query<(&mut Text, &StatusLabel)>,
) {
    if let Ok(player) = player_query.get_single() {
        let health_percent = player.health / player.max_health;
        let hunger_percent = player.hunger / player.max_hunger;
        let thirst_percent = player.thirst / player.max_thirst;

        for mut fill in fill_queries.p0().iter_mut() {
            fill.width = Val::Px(200.0 * health_percent);
        }

        for mut fill in fill_queries.p1().iter_mut() {
            fill.width = Val::Px(200.0 * hunger_percent);
        }

        for mut fill in fill_queries.p2().iter_mut() {
            fill.width = Val::Px(200.0 * thirst_percent);
        }

        for (mut text, label) in &mut status_label_query {
            match label.label_type {
                StatusLabelType::Health => {
                    text.0 = format!("Health: {:.0}/{:.0}", player.health, player.max_health);
                }
                StatusLabelType::Hunger => {
                    text.0 = format!("Hunger: {:.0}/{:.0}", player.hunger, player.max_hunger);
                }
                StatusLabelType::Thirst => {
                    text.0 = format!("Thirst: {:.0}/{:.0}", player.thirst, player.max_thirst);
                }
            }
        }
    }
}
