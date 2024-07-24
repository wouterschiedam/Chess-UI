use iced::{Background, Theme};

pub fn container_appearance(theme: &Theme) -> iced::widget::container::Appearance {
    iced::widget::container::Appearance {
        text_color: Some(iced::Color::from_rgb(132.0, 123.0, 124.0)),
        background: Some(Background::from(Theme::palette(theme).background)),
        ..Default::default()
    }
}
