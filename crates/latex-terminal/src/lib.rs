//! Terminal capability detection and image protocol primitives.

mod capabilities;
mod placement;
mod protocol;

pub use capabilities::FallbackReason;
pub use capabilities::TerminalBackend;
pub use capabilities::TerminalEnvironment;
pub use capabilities::TerminalSupport;
pub use capabilities::detect_terminal_support;
pub use placement::ImageDraw;
pub use placement::ImageRenderState;
pub use placement::ImageSource;
pub use placement::PlacementError;
pub use protocol::PlacementSize;
pub use protocol::ProtocolError;
pub use protocol::kitty_delete_image;
pub use protocol::kitty_transmit_png;
pub use protocol::kitty_transmit_png_file;
