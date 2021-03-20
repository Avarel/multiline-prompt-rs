use crate::{
    ext::RopeExt,
    keybindings::{Keybinding, NormalKeybinding},
    renderer::{full::FullRenderer, data::RenderData, Renderer},
};

use ropey::Rope;

use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode},
    Result,
};

/// Multiline abstraction around a terminal.
pub struct Editor<'w> {
    buf: Rope,
    pub cursor: Cursor,
    renderer: Box<dyn 'w + Renderer>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    /// Current line of the cursor.
    pub line: usize,
    /// Current index of the cursor.
    pub index: usize,
}

impl<'w> Editor<'w> {
    /// Create a builder for `MultilineTerm`.
    #[inline]
    pub fn builder<'b>() -> MultilineTermBuilder<'b> {
        MultilineTermBuilder::default()
    }

    pub fn line_count(&self) -> usize {
        self.buf.len_lines()
    }

    pub fn current_line_len(&self) -> usize {
        self.buf.line_trimmed(self.cursor.line).len_chars()
    }

    pub fn read_multiline(mut self) -> Result<String> {
        self.renderer
            .draw(RenderData::new(&self.buf, &self.cursor))?;
        self.renderer.flush()?;

        enable_raw_mode()?;
        loop {
            let take_more = NormalKeybinding.read(&mut self)?;

            self.renderer
                .redraw(RenderData::new(&self.buf, &self.cursor))?;
            self.renderer.flush()?;

            if take_more {
                continue;
            } else {
                break;
            }
        }

        // Clear the last empty useless line.
        self.renderer.clear_line()?;
        self.renderer.flush()?;

        disable_raw_mode()?;

        let buf = self.buf.to_string();

        Ok(buf)
    }

    pub fn clamp_cursor_index(&self) -> usize {
        self.cursor.index.min(self.current_line_len())
    }

    fn add_offset(z: usize, offset: isize) -> usize {
        if offset > 0 {
            z + offset as usize
        } else {
            z - -offset as usize
        }
    }

    fn cursor_rope_idx(&self, offset: isize) -> usize {
        let idx = self.cursor.index;
        let line_start = self.buf.line_to_char(self.cursor.line);
        let z = line_start + idx;
        Self::add_offset(z, offset)
    }

    pub fn delete_char_at_cursor(&mut self, offset: isize) -> usize {
        let z = self.cursor_rope_idx(offset);
        self.buf.remove(z..z + 1);
        Self::add_offset(self.cursor.index, offset)
    }

    pub fn insert_char_at_cursor(&mut self, c: char) -> usize {
        let z = self.cursor_rope_idx(0);
        self.buf.insert_char(z, c);
        self.cursor.index + 1
    }

    pub fn push_line(&mut self, string: &str) {
        self.buf.insert_line(self.line_count(), string)
    }

    pub fn remove_line(&mut self, line_idx: usize) -> String {
        self.buf.remove_line(line_idx)
    }

    pub fn push_curr_line(&mut self, string: &str) {
        let line_end = self.buf.line_to_char(self.cursor.line + 1) - 1;
        self.buf.insert(line_end, &string)
    }

    pub fn split_line(&mut self, line_idx: usize, cursor_idx: usize) {
        let line_start = self.buf.line_to_char(line_idx);
        let z = line_start + cursor_idx;
        self.buf.insert_char(z, '\n');
    }
}

/// Builder struct for `MultilineTerm`.
#[derive(Default)]
pub struct MultilineTermBuilder<'w> {
    /// Initial buffer for the multiline terminal.
    init: String,
    /// Initial line that the cursor is supposed to be set at.
    line: usize,
    /// Initial index that the cursor is supposed to be set at.
    index: usize,
    /// The renderer.
    renderer: Option<Box<dyn 'w + Renderer>>,
}

impl<'w> MultilineTermBuilder<'w> {
    /// Set the buffer that the terminal will be initialized with.
    #[inline]
    pub fn initial_buffers(mut self, init: String) -> Self {
        self.init = init;
        self
    }

    /// Set what line the cursor will initially start at.
    #[inline]
    pub fn line(mut self, line: usize) -> Self {
        self.line = line;
        self
    }

    /// Set what index the cursor will initially start at.
    #[inline]
    pub fn index(mut self, index: usize) -> Self {
        self.index = index;
        self
    }

    pub fn renderer(mut self, renderer: impl 'w + Renderer) -> Self {
        self.renderer = Some(Box::new(renderer));
        self
    }

    pub fn build(self) -> Editor<'w> {
        Editor {
            buf: Rope::from(self.init),
            cursor: Cursor {
                line: self.line,
                index: self.index,
            },
            renderer: self
                .renderer
                .unwrap_or_else(|| Box::new(FullRenderer::default())),
        }
    }
}
