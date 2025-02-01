use std::{
    fmt,
    io::{self, stdout, Stdout, Write as _},
};

use crossterm::{
    cursor::{self, MoveToNextLine},
    style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, ClearType},
    QueueableCommand,
};

pub trait EditorOutputAdapter: fmt::Write {
    fn clear(&mut self) -> io::Result<()>;

    fn cursor(&self) -> Cursor;

    fn move_cursor_to(&mut self, x: u16, y: u16) -> io::Result<()>;

    fn color(
        &mut self,
        color: Color,
        f: impl FnOnce(&mut Self) -> fmt::Result,
    ) -> anyhow::Result<()>;

    fn attribute(
        &mut self,
        attribute: Attribute,
        f: impl FnOnce(&mut Self) -> anyhow::Result<()>,
    ) -> anyhow::Result<()>;

    fn flush(&mut self) -> io::Result<()>;
}

#[derive(Debug)]
pub struct DebugOutputAdapter;

impl EditorOutputAdapter for DebugOutputAdapter {
    fn clear(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn cursor(&self) -> Cursor {
        Cursor { inner: [0; 2] }
    }

    fn move_cursor_to(&mut self, _: u16, _: u16) -> io::Result<()> {
        Ok(())
    }

    fn color(
        &mut self,
        _: Color,
        f: impl FnOnce(&mut Self) -> fmt::Result,
    ) -> anyhow::Result<()> {
        f(self)?;
        Ok(())
    }

    fn attribute(
        &mut self,
        _: Attribute,
        f: impl FnOnce(&mut Self) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        f(self)?;
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl fmt::Write for DebugOutputAdapter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        print!("{s}");
        Ok(())
    }
}

/// # Adapter between the renderer and the terminal
///
/// Unfortunately, terminals are an ancient technology and suck very badly. As a
/// result, writing to the terminal directly turned out to be impractical.
///
/// The specific problem encountered, was that determining the cursor position
/// can't be done without causing a flush, which leads to visual artifacts when
/// then resuming the rendering. As a result, we at least need something to
/// track the cursor position throughout the render. Hence this adapter.
pub struct RawTerminalAdapter {
    w: Stdout,
    cursor: [u16; 2],
}

impl RawTerminalAdapter {
    pub fn new() -> anyhow::Result<Self> {
        // Nothing forces us to enable raw mode right here. It's also tied to
        // input, so we could enable it there.
        //
        // It is very important, however, that we _disable_ it consistently,
        // depending on where we enabled it. Otherwise the terminal will remain
        // in raw mode after the application exited.
        //
        // We are taking care of that here, by disabling raw mode in the `Drop`
        // implementation of this type. So raw mode is bound to its lifetime.
        terminal::enable_raw_mode()?;

        Ok(Self {
            w: stdout(),
            cursor: [0, 0],
        })
    }

    fn write(&mut self, s: &str) -> io::Result<()> {
        for ch in s.chars() {
            if ch == '\n' {
                self.w.queue(MoveToNextLine(1))?;

                self.cursor[0] = 0;
                self.cursor[1] += 1;
            } else {
                let mut buf = [0; 4];
                self.w.write_all(ch.encode_utf8(&mut buf).as_bytes())?;

                assert!(
                    ch.is_ascii(),
                    "Editor input adapter only accepts ASCII characters.",
                );
                self.cursor[0] += 1;
            }
        }

        Ok(())
    }
}

impl EditorOutputAdapter for RawTerminalAdapter {
    fn clear(&mut self) -> io::Result<()> {
        self.w.queue(terminal::Clear(ClearType::All))?;
        self.move_cursor_to(0, 0)?;

        Ok(())
    }

    fn cursor(&self) -> Cursor {
        Cursor { inner: self.cursor }
    }

    fn move_cursor_to(&mut self, x: u16, y: u16) -> io::Result<()> {
        self.w.queue(cursor::MoveTo(x, y))?;
        self.cursor = [x, y];
        Ok(())
    }

    fn color(
        &mut self,
        color: Color,
        f: impl FnOnce(&mut Self) -> fmt::Result,
    ) -> anyhow::Result<()> {
        self.w.queue(SetForegroundColor(color))?;
        f(self)?;
        self.w.queue(ResetColor)?;

        Ok(())
    }

    fn attribute(
        &mut self,
        attribute: Attribute,
        f: impl FnOnce(&mut Self) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        self.w.queue(SetAttribute(attribute))?;
        f(self)?;
        self.w.queue(SetAttribute(Attribute::Reset))?;

        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.w.flush()
    }
}

impl fmt::Write for RawTerminalAdapter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s).map_err(|_| fmt::Error)
    }
}

impl Drop for RawTerminalAdapter {
    fn drop(&mut self) {
        // Nothing we can do about potential errors here.

        // If we don't clear the screen, the terminal is going to draw the
        // prompt over our remaining output, depending on where the cursor
        // happened to be.
        if let Err(err) = self.clear().and_then(|()| self.flush()) {
            println!("Failed to clear screen on shutdown: {err}");
        }

        let _ = terminal::disable_raw_mode();
    }
}

pub struct Cursor {
    pub inner: [u16; 2],
}

impl Cursor {
    pub fn move_right(self, offset: usize) -> Self {
        let [x, y] = self.inner;

        let x = {
            let x: usize = x.into();
            let x = x.saturating_add(offset);
            let x: u16 = x.try_into().unwrap_or(u16::MAX);
            x
        };

        Cursor { inner: [x, y] }
    }
}
