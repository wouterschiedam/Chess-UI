use iced::widget::container;
use iced::{Background, Color};

pub struct FloatingStyle;

impl container::StyleSheet for FloatingStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.7))),
            border_radius: 5.0.into(),
            ..container::Appearance::default()
        }
    }
}

