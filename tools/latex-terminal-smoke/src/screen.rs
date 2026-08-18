//! Alternate screen lifecycle for manual terminal acceptance.

use std::io::{self, Write as _};
use std::num::NonZeroU32;

use latex_terminal::kitty_delete_image;

const IMAGE_ID: NonZeroU32 = NonZeroU32::new(0xC0E2).expect("image identifier is nonzero");

pub(crate) struct ScreenSession {
    stdout: io::Stdout,
}

impl ScreenSession {
    pub(crate) fn start() -> Result<Self, io::Error> {
        let mut session = Self {
            stdout: io::stdout(),
        };
        session.stdout.write_all(b"\x1b[?1049h\x1b[2J\x1b[?25l")?;
        session.stdout.flush()?;
        Ok(session)
    }

    pub(crate) fn draw(&mut self, command: &[u8]) -> Result<(), io::Error> {
        self.stdout.write_all(command)?;
        self.stdout.flush()
    }
}

impl Drop for ScreenSession {
    fn drop(&mut self) {
        let _ = self
            .stdout
            .write_all(kitty_delete_image(IMAGE_ID).as_bytes());
        let _ = self.stdout.write_all(b"\x1b[?25h\x1b[?1049l");
        let _ = self.stdout.flush();
    }
}
