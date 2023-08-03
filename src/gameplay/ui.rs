use super::shared::Counter;
use super::shared::Health;
use super::shared::Name;
use bevy::prelude::*;

// General way of linking a game object to an UI object
#[derive(Component)]
pub struct Link(pub Entity);

// Marker of UI items that exist during gameplay
#[derive(Component)]
pub struct GameplayUI;

// Trait bound for progress bars such as a health bar
pub trait ProgressBar {}

// Trair for any text UI element that updates based on the data from
// a Counter component of some entity
pub trait UpdatingText {
    // Which component should be queried for the actual data
    type DataHolder: Component + Counter;

    // Unformatted text
    fn original(&self) -> String;
    // Text section to be modified. By default 0.
    fn section(&self) -> usize {
        0
    }
    // Which entity has the DataHolder component
    fn entity(&self) -> Entity;
}

#[derive(Component)]
pub enum ObjectType {
    Enemy,
    Player,
    Neutral,
}

// UI list of UpdatingText and others
// Acts as a root node for the elements in the ECS.
#[derive(Component, Debug)]
pub struct StatsList {
    // Where on the screen should the list be created at
    position: UiRect,
    // How many elements currently in the list
    length: usize,
    // Extra UI space between elements in the list
    padding: Val,
}
impl StatsList {
    pub fn new(position: UiRect, padding: Val) -> Self {
        Self {
            position,
            length: 0,
            padding,
        }
    }
}

// Create a StatsList at 30px off the top and right of the screen with a padding of 20px
pub fn create_stats_list(mut commands: Commands) {
    let position = UiRect {
        top: Val::Px(30.0),
        right: Val::Px(30.0),
        ..default()
    };
    let list = StatsList::new(position, Val::Px(20.0));

    // The list itself should be invisible, absolutely positioned, with elements being added
    // in a column, with the latest to be added being at the bottom of the list.
    commands.spawn((
        NodeBundle {
            style: Style {
                top: Val::Px(30.0),
                right: Val::Px(30.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        },
        list,
        GameplayUI,
    ));
}

// Add a counter to the stats list
pub fn create_counter<T: UpdatingText + Component>(
    commands: &mut Commands,
    list: &mut Query<(Entity, &mut StatsList)>,
    assets: &AssetServer,
    text: T,
) {
    // Retrieve the stats list from the world
    let (list_entity, list) = list
        .get_single_mut()
        .expect("None or more than 1 stats list was found.");
    // Calculate where to spawn the counter relative to the list
    let position = UiRect::top(list.padding * list.length as f32);

    // Add counter as a child of the list
    commands.entity(list_entity).with_children(|parent| {
        parent
            .spawn(
                TextBundle::from_section(
                    text.original(),
                    TextStyle {
                        font: assets.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 22.0,
                        color: Color::WHITE,
                    },
                )
                .with_text_alignment(TextAlignment::Left)
                .with_style(Style {
                    //align_self: AlignSelf::FlexStart,
                    position_type: PositionType::Relative,
                    margin: position,
                    ..default()
                }),
            )
            .insert(GameplayUI)
            .insert(text);
    });
}

// Add a health bar to the screen
pub fn create_health_bar<T: ProgressBar + Component>(
    commands: &mut Commands,
    assets: &AssetServer,
    name: Name,
    kind: ObjectType,
    health_bar_component: T,
) -> Entity {
    // Choose where the health bar spawn depending
    // on whether its the enemy's or the player's
    let position = match kind {
        ObjectType::Enemy => Style {
            top: Val::Px(30.0),
            left: Val::Px(30.0),
            ..default()
        },
        ObjectType::Player => Style {
            bottom: Val::Px(30.0),
            right: Val::Px(30.0),
            ..default()
        },
        ObjectType::Neutral => Style {
            top: Val::Px(30.0),
            right: Val::Px(30.0),
            ..default()
        },
    };

    // Create the text above the health bar
    let mut binding = commands.spawn(
        TextBundle::from_section(
            name.0.unwrap_or("").to_owned(),
            TextStyle {
                font: assets.load("fonts/FiraSans-Bold.ttf"),
                font_size: 22.0,
                color: Color::WHITE,
            },
        )
        .with_text_alignment(TextAlignment::Left)
        .with_style(Style {
            //align_self: AlignSelf::FlexStart,
            position_type: PositionType::Absolute,
            ..position
        }),
    );
    // The background of the bar
    binding
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        height: Val::Px(16.0),
                        top: Val::Px(20.0),
                        position_type: PositionType::Relative,
                        ..default()
                    },
                    background_color: Color::rgb(0.7, 0.7, 0.7).into(),
                    ..default()
                })
                // The inner part of the health bar
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                height: Val::Percent(80.0),
                                top: Val::Px(4.0),
                                bottom: Val::Px(4.0),
                                left: Val::Px(4.0),
                                right: Val::Px(4.0),
                                position_type: PositionType::Relative,
                                justify_content: JustifyContent::Center,
                                align_self: AlignSelf::Auto,
                                ..default()
                            },
                            background_color: Color::rgb(0.1, 0.8, 0.1).into(),
                            ..default()
                        })
                        .insert(health_bar_component);
                });
        })
        .insert(GameplayUI);

    binding.id()
}

// Change the heatlh bar size and colour based on the entity's health
pub fn update_health_bar<B: Component + ProgressBar, C: Component>(
    mut health_bars: Query<(&mut BackgroundColor, &mut Style), With<B>>,
    health: Query<&Health, With<C>>,
) {
    // Get the real health
    if let Ok(health) = health.get_single() {
        let fraction = health.current / health.total;
        for (mut bar_color, mut bar_style) in &mut health_bars {
            // Update bar size with percentage of total entity health
            bar_style.width = Val::Percent(fraction * 100.0);
            // Make the bar red when under 25% health
            if fraction <= 0.25 {
                bar_color.0 = Color::RED;
            }
        }
    }
}

// Update the text for a counter with the actual real-time data
pub fn update_counter_ui<T>(
    mut texts: Query<(&mut Text, &T)>,
    counter: Query<&<T as UpdatingText>::DataHolder>,
) where
    T: Component + UpdatingText,
{
    for (mut real_text, updating_text) in texts.iter_mut() {
        if let Ok(data) = counter.get(updating_text.entity()) {
            real_text.sections[updating_text.section()].value =
                format!("{} {}", updating_text.original().as_str(), data.get());
        }
    }
}
