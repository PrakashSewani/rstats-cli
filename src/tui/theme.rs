use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;
use std::{
    str::FromStr,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeName {
    #[default]
    Dark,
    Light,
    Mono,
}

impl ThemeName {
    pub const ALL: [ThemeName; 3] = [ThemeName::Dark, ThemeName::Light, ThemeName::Mono];

    pub fn as_str(self) -> &'static str {
        match self {
            ThemeName::Dark => "dark",
            ThemeName::Light => "light",
            ThemeName::Mono => "mono",
        }
    }

    pub fn next(self) -> Self {
        match self {
            ThemeName::Dark => ThemeName::Light,
            ThemeName::Light => ThemeName::Mono,
            ThemeName::Mono => ThemeName::Dark,
        }
    }
}

impl FromStr for ThemeName {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "dark" => Ok(ThemeName::Dark),
            "light" => Ok(ThemeName::Light),
            "mono" => Ok(ThemeName::Mono),
            other => Err(format!("invalid theme `{other}` (expected dark, light, or mono)")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    pub title: Style,
    pub muted: Style,
    pub gauge: Style,
    pub warning: Style,
    pub critical: Style,
}

impl Palette {
    pub fn for_name(name: ThemeName) -> Self {
        match name {
            ThemeName::Dark => Self {
                title: Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                muted: Style::default().fg(Color::DarkGray),
                gauge: Style::default().fg(Color::Green),
                warning: Style::default().fg(Color::Yellow),
                critical: Style::default().fg(Color::Red),
            },
            ThemeName::Light => Self {
                title: Style::default().fg(Color::Rgb(38, 139, 210)).add_modifier(Modifier::BOLD),
                muted: Style::default().fg(Color::Rgb(101, 123, 131)),
                gauge: Style::default().fg(Color::Rgb(133, 153, 0)),
                warning: Style::default().fg(Color::Rgb(181, 137, 0)),
                critical: Style::default().fg(Color::Rgb(220, 50, 47)),
            },
            ThemeName::Mono => Self {
                title: Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                muted: Style::default().fg(Color::DarkGray),
                gauge: Style::default().fg(Color::White),
                warning: Style::default().fg(Color::Gray),
                critical: Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            },
        }
    }
}

static CURRENT_THEME: AtomicUsize = AtomicUsize::new(0);

pub fn set_theme(name: ThemeName) {
    let index = ThemeName::ALL.iter().position(|candidate| *candidate == name).unwrap_or(0);
    CURRENT_THEME.store(index, Ordering::Relaxed);
}

pub fn current_theme() -> ThemeName {
    ThemeName::ALL[CURRENT_THEME.load(Ordering::Relaxed).min(ThemeName::ALL.len() - 1)]
}

pub fn cycle_theme() -> ThemeName {
    let next = current_theme().next();
    set_theme(next);
    next
}

fn palette() -> Palette {
    Palette::for_name(current_theme())
}

pub fn title() -> Style {
    palette().title
}

pub fn muted() -> Style {
    palette().muted
}

pub fn gauge() -> Style {
    palette().gauge
}

pub fn warning() -> Style {
    palette().warning
}

pub fn critical() -> Style {
    palette().critical
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_theme_names() {
        assert_eq!("dark".parse::<ThemeName>(), Ok(ThemeName::Dark));
        assert_eq!("light".parse::<ThemeName>(), Ok(ThemeName::Light));
        assert_eq!("mono".parse::<ThemeName>(), Ok(ThemeName::Mono));
        assert!("banana".parse::<ThemeName>().is_err());
    }

    #[test]
    fn cycles_through_all_themes_and_wraps() {
        assert_eq!(ThemeName::Dark.next(), ThemeName::Light);
        assert_eq!(ThemeName::Light.next(), ThemeName::Mono);
        assert_eq!(ThemeName::Mono.next(), ThemeName::Dark);
    }

    #[test]
    fn palettes_provide_distinct_styles() {
        let dark = Palette::for_name(ThemeName::Dark);
        let light = Palette::for_name(ThemeName::Light);
        let mono = Palette::for_name(ThemeName::Mono);
        assert_ne!(dark.title, light.title);
        assert_ne!(dark.gauge, light.gauge);
        assert_ne!(dark.critical, mono.critical);
        assert_ne!(light.muted, mono.muted);
    }

    #[test]
    fn theme_helpers_follow_the_selected_theme() {
        set_theme(ThemeName::Dark);
        assert_eq!(title(), Palette::for_name(ThemeName::Dark).title);
        assert_eq!(cycle_theme(), ThemeName::Light);
        assert_eq!(current_theme(), ThemeName::Light);
        assert_eq!(gauge(), Palette::for_name(ThemeName::Light).gauge);
        set_theme(ThemeName::Dark);
    }
}
