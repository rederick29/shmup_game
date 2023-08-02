use crate::GameOptions;

use super::Action;
use super::InOptionsMenu;
use bevy::prelude::*;

// Markers for options text elements
#[derive(Debug, Clone, Copy, Component)]
pub enum OptionText {
    Volume,
    InvertFocus,
}

pub fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    let font: Handle<Font> = assets.load("fonts/FiraSans-Bold.ttf");
    let button_style = Style {
        size: Size::new(Val::Px(120.0), Val::Px(40.0)),
        margin: UiRect::all(Val::Px(10.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let text_style = TextStyle {
        font: font.clone(),
        font_size: 36.0,
        color: crate::ui::TEXT_COLOUR,
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(95.0), Val::Percent(95.0)),
                    position: UiRect {
                        top: Val::Px(15.0),
                        left: Val::Px(15.0),
                        ..default()
                    },
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    ..default()
                },
                ..default()
            },
            InOptionsMenu,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Options",
                TextStyle {
                    font_size: 40.0,
                    font: font.clone(),
                    color: crate::ui::TEXT_COLOUR,
                },
            ));

            for (action, text, component, alternate) in [
                (
                    Action::InvertFocus,
                    "Switch",
                    Some(OptionText::InvertFocus),
                    None,
                ),
                (
                    Action::Sound,
                    "Volume",
                    Some(OptionText::Volume),
                    Some(setup_volume_buttons),
                ),
                (Action::GoToMenu, "Back", None, None),
            ] {
                if let Some(alternative_setup) = alternate {
                    alternative_setup(parent, &text_style, &button_style);
                    continue;
                }
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_wrap: FlexWrap::NoWrap,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            size: Size::new(Val::Percent(98.0), Val::Percent(10.0)),
                            margin: UiRect {
                                top: Val::Px(25.0),
                                left: Val::Px(15.0),
                                ..default()
                            },
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        let mut option_name =
                            parent.spawn(TextBundle::from_section(text, text_style.clone()));
                        if let Some(component) = component {
                            option_name.insert(component);
                        }
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    background_color: crate::ui::BUTTON_BASE.into(),
                                    ..default()
                                },
                                action,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(text, text_style.clone()));
                            });
                    });
            }
        });
}

fn setup_volume_buttons(parent: &mut ChildBuilder, text_style: &TextStyle, button_style: &Style) {
    let mut custom_style = button_style.clone();
    custom_style.size.width = button_style.size.width / 2.0;

    parent
        .spawn(NodeBundle {
            style: Style {
                flex_wrap: FlexWrap::NoWrap,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                size: Size::new(Val::Percent(98.0), Val::Percent(10.0)),
                margin: UiRect {
                    top: Val::Px(25.0),
                    left: Val::Px(15.0),
                    ..default()
                },
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section("Volume", text_style.clone()),
                OptionText::Volume,
            ));
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_wrap: FlexWrap::NoWrap,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            ButtonBundle {
                                style: custom_style.clone(),
                                background_color: crate::ui::BUTTON_BASE.into(),
                                ..default()
                            },
                            Action::VolumeUp,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section("+", text_style.clone()));
                        });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: custom_style.clone(),
                                background_color: crate::ui::BUTTON_BASE.into(),
                                ..default()
                            },
                            Action::VolumeDown,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section("-", text_style.clone()));
                        });
                });
        });
}

pub fn update_option_text(mut query: Query<(&mut Text, &OptionText)>, options: Res<GameOptions>) {
    for (mut text, option_kind) in &mut query {
        match option_kind {
            OptionText::Volume => {
                text.sections[0].value = format!("Volume: {:.0}", options.get_volume() * 10.);
            }
            OptionText::InvertFocus => {
                text.sections[0].value = if options.get_focus() {
                    "Focus Mode: Inverted".to_string()
                } else {
                    "Focus Mode: Normal".to_string()
                }
            }
        }
    }
}
