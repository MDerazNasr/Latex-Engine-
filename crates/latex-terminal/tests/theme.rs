#![doc = "Terminal math theme resolution tests."]

use latex_render_core::Rgba;
use latex_terminal::{TerminalAppearance, ThemeMode, resolve_math_theme};

#[test]
fn auto_uses_the_host_hint_and_defaults_to_dark() {
    for appearance in [
        TerminalAppearance::default(),
        TerminalAppearance {
            dark_background: Some(true),
        },
    ] {
        let theme = resolve_math_theme(ThemeMode::Auto, appearance);
        assert_eq!(theme.mode(), ThemeMode::Dark);
        assert_eq!(theme.foreground(), Rgba::opaque(230, 237, 243));
        assert_eq!(theme.background(), None);
    }

    let light = resolve_math_theme(
        ThemeMode::Auto,
        TerminalAppearance {
            dark_background: Some(false),
        },
    );
    assert_eq!(light.mode(), ThemeMode::Light);
    assert_eq!(light.foreground(), Rgba::opaque(17, 24, 39));
    assert_eq!(light.background(), None);
}

#[test]
fn explicit_modes_override_the_host_hint() {
    let dark = resolve_math_theme(
        ThemeMode::Dark,
        TerminalAppearance {
            dark_background: Some(false),
        },
    );
    let light = resolve_math_theme(
        ThemeMode::Light,
        TerminalAppearance {
            dark_background: Some(true),
        },
    );

    assert_eq!(dark.mode(), ThemeMode::Dark);
    assert_eq!(dark.foreground(), Rgba::opaque(230, 237, 243));
    assert_eq!(light.mode(), ThemeMode::Light);
    assert_eq!(light.foreground(), Rgba::opaque(17, 24, 39));
}

#[test]
fn default_theme_colors_have_high_reference_contrast() {
    let dark = resolve_math_theme(ThemeMode::Dark, TerminalAppearance::default());
    let light = resolve_math_theme(ThemeMode::Light, TerminalAppearance::default());
    let dark_background = Rgba::opaque(13, 17, 23);
    let light_background = Rgba::opaque(255, 255, 255);

    assert!(contrast_ratio(dark.foreground(), dark_background) >= 7.0);
    assert!(contrast_ratio(light.foreground(), light_background) >= 7.0);
}

#[test]
fn diagnostic_names_are_stable_configuration_values() {
    assert_eq!(ThemeMode::Auto.diagnostic_name(), "auto");
    assert_eq!(ThemeMode::Light.diagnostic_name(), "light");
    assert_eq!(ThemeMode::Dark.diagnostic_name(), "dark");
}

fn contrast_ratio(first: Rgba, second: Rgba) -> f64 {
    let first = luminance(first);
    let second = luminance(second);
    let lighter = first.max(second);
    let darker = first.min(second);
    (lighter + 0.05) / (darker + 0.05)
}

fn luminance(color: Rgba) -> f64 {
    let linear = |channel: u8| {
        let channel = f64::from(channel) / 255.0;
        if channel <= 0.04045 {
            channel / 12.92
        } else {
            ((channel + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(color.red) + 0.7152 * linear(color.green) + 0.0722 * linear(color.blue)
}
