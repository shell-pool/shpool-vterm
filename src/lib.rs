use std::collections::VecDeque;

/// A representation of a terminal.
///
/// The terminal state is represented as a dequeue of lines which
/// might be arbitrarily long. If a new line is added to the terminal
/// when the queue is full, the oldest line will be discarded. It is
/// importaint to understand that lines in the terminal state do not
/// corrispond to physical lines as they would appear on the screen.
/// A single logical line could wrap zero or more times, causing it
/// to take up one or more physical lines when displayed.
///
/// TODO: Will I need to have a "grid view" layer that allows indexing
/// into the logical data as if it were laid out in a grid? I think this
/// may be required to handle certain control sequences. Might need to
/// use some sort of weird tree structure that notes the grid line that
/// each logical line starts on and then allows you to find the logical
/// line for each grid line.
#[derive(Debug, Clone)]
pub struct Term {
    size: Size,
    lines: VecDeque<LogicalLine>,
}

impl Term {
    /// Create a new terminal with the given width and height.
    ///
    /// Note that width will only be used when generated output
    /// to determine where wrapping should be place.
    pub fn new(size: Size) -> Self {
        Term {
            size,
            lines: VecDeque::with_capacity(size.height),
        }
    }

    /// Get the current terminal size.
    pub fn size(&self) -> Size {
        self.size
    }

    /// Set the terminal size.
    pub fn set_size(&mut self, size: Size) {
        self.size = size;
    }

    /// Process the given chunk of input. This should be the data read off
    /// a pty running a shell.
    pub fn process(&mut self, _buf: &[u8]) {
        // TODO: stub
    }

    /// Get the current contents of the terminal encoded via terminal
    /// escape sequences. The contents buffer will be prefixed with
    /// a reset code, so inputing this to any terminal emulator will
    /// reset the emulator to the contents of this Term instance.
    ///
    /// The size parameter asks for the contents to be formated for
    /// a terminal of the given size. This is mostly useful if the
    /// virtual terminal has more lines than are desired.
    pub fn contents(&self, size: Option<Size>) -> Vec<u8> {
        // TODO: stub
        vec![]
    }

    //
    // Internal helpers
    //

    /// Push a new line, making sure old lines are discared.
    fn push_line(&mut self, line: LogicalLine) {
        // TODO: if lines maintain a mapping to their grid position,
        // this would invalidate that mapping. It should either be
        // recomputed or if we are lazy about it, invalidated.
        self.lines.truncate(self.size.height - 1);
        self.lines.push_front(line);
    }
}

/// The size of the terminal.
#[derive(Debug, Clone, Copy)]
pub struct Size {
    width: usize,
    height: usize,
}

/// A logical line. May be arbitrarily long and will only get wrapped
/// when transforming the term into a grid view.
#[derive(Debug, Clone)]
struct LogicalLine {
}


#[cfg(test)]
mod tests {
    use super::*;
}
