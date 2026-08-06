//! Color values shared by renderer implementations.

/// A red, green, blue, and alpha color value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Rgba {
    /// Red channel.
    pub red: u8,
    /// Green channel.
    pub green: u8,
    /// Blue channel.
    pub blue: u8,
    /// Alpha channel.
    pub alpha: u8,
}

impl Rgba {
    /// Creates a color from four channel values.
    pub const fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// Creates a fully opaque color.
    pub const fn opaque(red: u8, green: u8, blue: u8) -> Self {
        Self::new(red, green, blue, u8::MAX)
    }

    /// Returns whether the color is fully opaque.
    pub const fn is_opaque(self) -> bool {
        self.alpha == u8::MAX
    }

    /// Returns whether the color is fully transparent.
    pub const fn is_transparent(self) -> bool {
        self.alpha == 0
    }

    /// Formats the color as an opaque CSS hexadecimal value.
    pub fn to_rgb_hex(self) -> Option<String> {
        self.is_opaque()
            .then(|| format!("#{:02x}{:02x}{:02x}", self.red, self.green, self.blue))
    }

    pub(crate) const fn channels(self) -> [u8; 4] {
        [self.red, self.green, self.blue, self.alpha]
    }
}
