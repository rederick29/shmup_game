use bevy::prelude::*;

// Consistent colour scheme for buttons and text throughout the game
pub const BUTTON_BASE: Color = Color::rgb(0.2, 0.2, 0.2);
pub const BUTTON_HOVER: Color = Color::rgb(0.45, 0.35, 0.35);
pub const BUTTON_PRESS: Color = Color::rgb(0.75, 0.55, 0.55);
pub const TEXT_COLOUR: Color = Color::rgb(0.9, 0.9, 0.9);

// Change the colour of buttons when hovered over or clicked on
#[allow(clippy::type_complexity)]
pub fn colour_buttons(
    mut interaction: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut colour) in interaction.iter_mut() {
        *colour = match *interaction {
            Interaction::Pressed => BUTTON_PRESS.into(),
            Interaction::Hovered => BUTTON_HOVER.into(),
            Interaction::None => BUTTON_BASE.into(),
        }
    }
}

// Change the colour of text marked as such
pub fn animate_text<T: Component>(time: Res<Time>, mut query: Query<&mut Text, With<T>>) {
    for mut text in query.iter_mut() {
        let t = time.elapsed_seconds();
        text.sections[0].style.color = Color::Rgba {
            // sin results in a value between -1 and +1, yet RGB(A) does not
            // take negative values. Assuming minimum value of -1, the
            // following makes it >= 0. -1.0 / 2.0 = -0.5, -0.5 + 0.5 = 0.0.
            red: (t).sin() / 2.0 + 0.5,
            green: (t + 1.9).sin() / 2.0 + 0.5,
            blue: (t + 0.8).sin() / 2.0 + 0.5,
            alpha: 1.0,
        };
    }
}
