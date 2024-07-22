// use iced::widget::button::{Appearance, StyleSheet};
// use iced::{Background, BorderRadius, Color};
//
// #[derive(Debug, Clone, Copy)]
// pub struct LightSquare {
//     pub active: Appearance,
//     pub hovered: Appearance,
//     pub pressed: Appearance,
//     pub disabled: Appearance,
//     pub current_state: ContainerState,
// }
//
// #[derive(Default, Copy, Clone, Debug)]
// pub enum ContainerState {
//     #[default]
//     Active,
//     Hovered,
//     Pressed,
//     Disabled,
// }
//
// impl LightSquare {
//     pub fn new() -> Self {
//         let default = Appearance {
//             shadow_offset: Default::default(),
//             background: None,
//             border_radius: Default::default(),
//             border_width: 0.0,
//             border_color: Default::default(),
//             text_color: Default::default(),
//         };
//         Self {
//             active: default,
//             hovered: default,
//             pressed: default,
//             disabled: default,
//             current_state: Default::default(),
//         }
//     }
//
//     pub fn primary(theme: &iced::Theme) -> Self {
//         Self {
//             active: theme.active(&iced::theme::Button::Primary),
//             hovered: theme.hovered(&iced::theme::Button::Primary),
//             pressed: theme.pressed(&iced::theme::Button::Primary),
//             disabled: theme.disabled(&iced::theme::Button::Primary),
//             current_state: Default::default(),
//         }
//     }
//
//     pub fn secondary(theme: &iced::Theme) -> Self {
//         Self {
//             active: theme.active(&iced::theme::Button::Secondary),
//             hovered: theme.hovered(&iced::theme::Button::Secondary),
//             pressed: theme.pressed(&iced::theme::Button::Secondary),
//             disabled: theme.disabled(&iced::theme::Button::Secondary),
//             current_state: Default::default(),
//         }
//     }
//
//     pub fn destructive(theme: &iced::Theme) -> Self {
//         Self {
//             active: theme.active(&iced::theme::Button::Destructive),
//             hovered: theme.hovered(&iced::theme::Button::Destructive),
//             pressed: theme.pressed(&iced::theme::Button::Destructive),
//             disabled: theme.disabled(&iced::theme::Button::Destructive),
//             current_state: Default::default(),
//         }
//     }
//
//     pub fn positive(theme: &iced::Theme) -> Self {
//         Self {
//             active: theme.active(&iced::theme::Button::Positive),
//             hovered: theme.hovered(&iced::theme::Button::Positive),
//             pressed: theme.pressed(&iced::theme::Button::Positive),
//             disabled: theme.disabled(&iced::theme::Button::Positive),
//             current_state: Default::default(),
//         }
//     }
//
//     pub fn text(theme: &iced::Theme) -> Self {
//         Self {
//             active: theme.active(&iced::theme::Button::Text),
//             hovered: theme.hovered(&iced::theme::Button::Text),
//             pressed: theme.pressed(&iced::theme::Button::Text),
//             disabled: theme.disabled(&iced::theme::Button::Text),
//             current_state: Default::default(),
//         }
//     }
//
//     pub fn active(mut self) -> Self {
//         self.current_state = ContainerState::Active;
//         self
//     }
//
//     pub fn hovered(mut self) -> Self {
//         self.current_state = ContainerState::Hovered;
//         self
//     }
//
//     pub fn pressed(mut self) -> Self {
//         self.current_state = ContainerState::Pressed;
//         self
//     }
//
//     pub fn disabled(mut self) -> Self {
//         self.current_state = ContainerState::Disabled;
//         self
//     }
//
//     pub fn background(mut self, background: Option<Background>) -> Self {
//         match self.current_state {
//             ContainerState::Active => self.active.background = background,
//             ContainerState::Hovered => self.hovered.background = background,
//             ContainerState::Pressed => self.pressed.background = background,
//             ContainerState::Disabled => self.disabled.background = background,
//         }
//         self
//     }
//
//     pub fn text_color(mut self, color: Color) -> Self {
//         match self.current_state {
//             ContainerState::Active => self.active.text_color = color,
//             ContainerState::Hovered => self.hovered.text_color = color,
//             ContainerState::Pressed => self.pressed.text_color = color,
//             ContainerState::Disabled => self.disabled.text_color = color,
//         }
//         self
//     }
//
//     pub fn background_color(mut self, color: Color) -> Self {
//         match self.current_state {
//             ContainerState::Active => self.active.background = Some(Background::Color(color)),
//             ContainerState::Hovered => self.hovered.background = Some(Background::Color(color)),
//             ContainerState::Pressed => self.pressed.background = Some(Background::Color(color)),
//             ContainerState::Disabled => self.disabled.background = Some(Background::Color(color)),
//         }
//         self
//     }
//
//     pub fn border_radius(mut self, radius: BorderRadius) -> Self {
//         match self.current_state {
//             ContainerState::Active => self.active.border_radius = radius,
//             ContainerState::Hovered => self.hovered.border_radius = radius,
//             ContainerState::Pressed => self.pressed.border_radius = radius,
//             ContainerState::Disabled => self.disabled.border_radius = radius,
//         }
//         self
//     }
//
//     pub fn border_width(mut self, width: f32) -> Self {
//         match self.current_state {
//             ContainerState::Active => self.active.border_width = width,
//             ContainerState::Hovered => self.hovered.border_width = width,
//             ContainerState::Pressed => self.pressed.border_width = width,
//             ContainerState::Disabled => self.disabled.border_width = width,
//         }
//         self
//     }
//
//     pub fn border_color(mut self, color: Color) -> Self {
//         match self.current_state {
//             ContainerState::Active => self.active.border_color = color,
//             ContainerState::Hovered => self.hovered.border_color = color,
//             ContainerState::Pressed => self.pressed.border_color = color,
//             ContainerState::Disabled => self.disabled.border_color = color,
//         }
//         self
//     }
//
//     pub fn shadow_offset(mut self, offset: iced::Vector) -> Self {
//         match self.current_state {
//             ContainerState::Active => self.active.shadow_offset = offset,
//             ContainerState::Hovered => self.hovered.shadow_offset = offset,
//             ContainerState::Pressed => self.pressed.shadow_offset = offset,
//             ContainerState::Disabled => self.disabled.shadow_offset = offset,
//         }
//         self
//     }
//
//     pub fn as_custom(&self) -> iced::theme::Button {
//         iced::theme::Button::Custom(Box::new(*self))
//     }
// }
//
// impl StyleSheet for LightSquare {
//     type Style = iced::Theme;
//
//     fn active(&self, _style: &Self::Style) -> Appearance {
//         self.active
//     }
//
//     fn hovered(&self, _style: &Self::Style) -> Appearance {
//         self.hovered
//     }
//
//     fn pressed(&self, _style: &Self::Style) -> Appearance {
//         self.pressed
//     }
//
//     fn disabled(&self, _style: &Self::Style) -> Appearance {
//         self.disabled
//     }
// }

use iced::Color;

#[derive(Debug)]
pub enum Squares {
    Even(LightSquare),
    Odd(DarkSquare),
}

#[derive(Default, Debug)]
pub struct Terminal;

impl iced::widget::container::StyleSheet for Terminal {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::BLACK)),
            text_color: Some(Color::WHITE),
            border_width: 1.0,
            border_color: Color::WHITE,
            ..Default::default()
        }
    }
}

#[derive(Default, Debug)]
pub struct LightSquare;

impl iced::widget::container::StyleSheet for LightSquare {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(
                0.639, 0.502, 0.329,
            ))),
            text_color: Some(Color::WHITE),
            ..Default::default()
        }
    }
}

#[derive(Default, Debug)]
pub struct DarkSquare;

impl iced::widget::container::StyleSheet for DarkSquare {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.91, 0.741, 0.529))),
            text_color: Some(Color::WHITE),
            ..Default::default()
        }
    }
}
