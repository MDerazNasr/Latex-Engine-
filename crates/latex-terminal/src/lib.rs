//! Terminal capability detection and image protocol primitives.

mod capabilities;
mod layout;
mod placement;
mod presenter;
mod protocol;

pub use capabilities::FallbackReason;
pub use capabilities::TerminalBackend;
pub use capabilities::TerminalEnvironment;
pub use capabilities::TerminalSupport;
pub use capabilities::detect_terminal_support;
pub use layout::ImageLayout;
pub use layout::LayoutError;
pub use layout::LayoutMode;
pub use layout::LayoutPolicy;
pub use layout::MathGeometry;
pub use layout::PixelRect;
pub use layout::TerminalGeometry;
pub use layout::layout_math;
pub use placement::ImageDraw;
pub use placement::ImageRenderState;
pub use placement::ImageSource;
pub use placement::PlacementError;
pub use presenter::PresentationError;
pub use presenter::PresentationJob;
pub use presenter::PublishOutcome;
pub use presenter::RasterizedPresentation;
pub use presenter::TerminalPresenter;
pub use presenter::rasterize_presentation;
pub use protocol::PlacementSize;
pub use protocol::ProtocolError;
pub use protocol::kitty_delete_image;
pub use protocol::kitty_transmit_png;
pub use protocol::kitty_transmit_png_file;
