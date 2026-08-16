//! Deterministic colors for transparent terminal math images.

use latex_render_core::Rgba;

const DARK_FOREGROUND: Rgba = Rgba::opaque(230, 237, 243);
const LIGHT_FOREGROUND: Rgba = Rgba::opaque(17, 24, 39);

/// User selection for terminal math appearance.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ThemeMode {
    /// Follow a reliable host hint and otherwise use the dark default.
    #[default]
    Auto,
    /// Use dark glyphs intended for a light terminal background.
    Light,
    /// Use light glyphs intended for a dark terminal background.
    Dark,
}

impl ThemeMode {
    /// Returns the stable name used by configuration and diagnostics.
    pub const fn diagnostic_name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}

/// Appearance information already known by the host terminal UI.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct TerminalAppearance {
    /// Whether the host knows that its background is dark.
    pub dark_background: Option<bool>,
}

/// Concrete colors and mode used for one renderer request.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ResolvedMathTheme {
    mode: ThemeMode,
    foreground: Rgba,
}

impl ResolvedMathTheme {
    /// Returns the resolved light or dark mode.
    pub const fn mode(self) -> ThemeMode {
        self.mode
    }

    /// Returns the opaque equation foreground.
    pub const fn foreground(self) -> Rgba {
        self.foreground
    }

    /// Returns the transparent background required for terminal blending.
    pub const fn background(self) -> Option<Rgba> {
        None
    }
}

/// Resolves explicit or automatic theme selection without terminal side effects.
pub const fn resolve_math_theme(
    selection: ThemeMode,
    appearance: TerminalAppearance,
) -> ResolvedMathTheme {
    let mode = match selection {
        ThemeMode::Auto => match appearance.dark_background {
            Some(false) => ThemeMode::Light,
            Some(true) | None => ThemeMode::Dark,
        },
        ThemeMode::Light => ThemeMode::Light,
        ThemeMode::Dark => ThemeMode::Dark,
    };
    let foreground = match mode {
        ThemeMode::Light => LIGHT_FOREGROUND,
        ThemeMode::Auto | ThemeMode::Dark => DARK_FOREGROUND,
    };
    ResolvedMathTheme { mode, foreground }
}
