use iced::{
    border, widget::{container}, Color, Theme
};

use iced::widget::button as iced_button;

pub fn container_background(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    let base = container::Style {
        background: iced::Background::Color(Color::from_rgb8(59, 64, 96)).into(),
        border: border::rounded(5),
        ..container::Style::default()
    };
    base
}

// This style was backported from 0.14, where it's available built in.
fn button_background(theme: &Theme, status: iced_button::Status) -> iced_button::Style {
    let palette = theme.extended_palette();
    let base = iced_button::Style {
        background: Some(palette.background.base.color.into()),
        text_color: palette.background.base.text,
        border: border::rounded(2),
        ..Default::default()
    };

    match status {
        iced_button::Status::Active => base,
        iced_button::Status::Pressed => iced_button::Style {
            background: Some(palette.background.strong.color.into()),
            ..base
        },
        iced_button::Status::Hovered => iced_button::Style {
            background: Some(palette.background.weak.color.into()),
            ..base
        },
        iced_button::Status::Disabled => iced_button::Style {
            background: base
                .background
                .map(|background| background.scale_alpha(0.5)),
            text_color: base.text_color.scale_alpha(0.5),
            ..base
        },
    }
}

#[derive(Debug, Clone, Copy)]
enum Link {
    Primary,
    Secondary,
}

pub fn link(text: &str) -> iced::widget::Button<Message> {
    iced_button(text).style(button_background)
}