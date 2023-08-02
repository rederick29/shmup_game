use super::Action;
use super::InMainMenu;
use bevy::prelude::*;

// Create the main menu
pub fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    let font: Handle<Font> = assets.load("fonts/FiraSans-Bold.ttf");
    let button_style = Style {
        size: Size::new(Val::Px(175.0), Val::Px(50.0)),
        margin: UiRect::all(Val::Px(15.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 40.0,
        color: crate::ui::TEXT_COLOUR,
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    margin: UiRect {
                        left: Val::Auto,
                        right: Val::Auto,
                        top: Val::Px(40.0),
                        bottom: Val::Auto,
                    },
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::CRIMSON.into(),
                ..default()
            },
            InMainMenu,
        ))
        .with_children(|parent| {
            // Display the game name
            parent.spawn(
                TextBundle::from_section(
                    env!("CARGO_PKG_NAME").to_string(),
                    TextStyle {
                        font: font.clone(),
                        font_size: 60.0,
                        color: crate::ui::TEXT_COLOUR,
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(50.0)),
                    ..default()
                }),
            );
        });
    // Create buttons list
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        right: Val::Px(20.0),
                        bottom: Val::Px(10.0),
                        ..default()
                    },
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::FlexStart,
                    ..default()
                },
                // Background colour of space around buttons
                background_color: Color::NONE.into(),
                ..default()
            },
            InMainMenu,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        style: button_style.clone(),
                        background_color: crate::ui::BUTTON_BASE.into(),
                        ..default()
                    },
                    Action::StartGameplay,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section("Play", text_style.clone()));
                });
            parent
                .spawn((
                    ButtonBundle {
                        style: button_style.clone(),
                        background_color: crate::ui::BUTTON_BASE.into(),
                        ..default()
                    },
                    Action::GoToOptions,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section("Settings", text_style.clone()));
                });
            parent
                .spawn((
                    ButtonBundle {
                        style: button_style.clone(),
                        background_color: crate::ui::BUTTON_BASE.into(),
                        ..default()
                    },
                    Action::Exit,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section("Quit", text_style.clone()));
                });
        });
}
