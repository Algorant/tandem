use std::io::{self, Write};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, BeginSynchronizedUpdate, EndSynchronizedUpdate,
        EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::CliError;

pub(super) struct TerminalSession {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl TerminalSession {
    pub(super) fn enter() -> Result<Self, CliError> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        if let Err(error) = execute!(stdout, EnterAlternateScreen, EnableMouseCapture) {
            let _ = disable_raw_mode();
            return Err(error.into());
        }

        let backend = CrosstermBackend::new(stdout);
        match Terminal::new(backend) {
            Ok(mut terminal) => {
                if let Err(error) = terminal.clear() {
                    restore_terminal(terminal.backend_mut());
                    return Err(error.into());
                }
                Ok(Self { terminal })
            }
            Err(error) => {
                let _ = disable_raw_mode();
                let mut stdout = io::stdout();
                let _ = leave_terminal(&mut stdout);
                Err(error.into())
            }
        }
    }

    pub(super) fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<io::Stdout>> {
        &mut self.terminal
    }

    pub(super) fn draw_synchronized<F>(&mut self, draw: F) -> Result<(), CliError>
    where
        F: FnOnce(&mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()>,
    {
        draw_synchronized_on(&mut self.terminal, draw)
    }

    pub(super) fn suspend_for_editor(&mut self) -> Result<(), CliError> {
        self.terminal.show_cursor()?;
        self.terminal.backend_mut().flush()?;
        disable_raw_mode()?;
        leave_terminal(self.terminal.backend_mut())?;
        Ok(())
    }

    pub(super) fn resume_after_editor(&mut self) -> Result<(), CliError> {
        enable_raw_mode()?;
        if let Err(error) = execute!(
            self.terminal.backend_mut(),
            EnterAlternateScreen,
            EnableMouseCapture
        ) {
            let _ = disable_raw_mode();
            return Err(error.into());
        }
        self.terminal.clear()?;
        Ok(())
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        restore_terminal(self.terminal.backend_mut());
        let _ = self.terminal.show_cursor();
    }
}

fn restore_terminal(backend: &mut CrosstermBackend<io::Stdout>) {
    let _ = disable_raw_mode();
    let _ = leave_terminal(backend);
}

fn draw_synchronized_on<W, F>(
    terminal: &mut Terminal<CrosstermBackend<W>>,
    draw: F,
) -> Result<(), CliError>
where
    W: Write,
    F: FnOnce(&mut Terminal<CrosstermBackend<W>>) -> io::Result<()>,
{
    execute!(terminal.backend_mut(), BeginSynchronizedUpdate)?;
    let draw_result = draw(terminal).map_err(CliError::from);
    let end_result = execute!(terminal.backend_mut(), EndSynchronizedUpdate);
    draw_result?;
    end_result.map_err(CliError::from)
}

fn leave_terminal(writer: &mut impl Write) -> io::Result<()> {
    execute!(writer, LeaveAlternateScreen, DisableMouseCapture)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;

    #[derive(Clone)]
    struct SharedWriter(Rc<RefCell<Vec<u8>>>);

    impl Write for SharedWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.borrow_mut().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn synchronized_update_emits_markers_around_frame_output() {
        let output = Rc::new(RefCell::new(Vec::new()));
        let writer = SharedWriter(Rc::clone(&output));
        let mut terminal = Terminal::new(CrosstermBackend::new(writer)).unwrap();
        draw_synchronized_on(&mut terminal, |terminal| {
            terminal.backend_mut().write_all(b"frame")
        })
        .unwrap();
        let output = String::from_utf8(output.borrow().clone()).unwrap();

        let begin = output.find("\x1b[?2026h").unwrap();
        let frame = output.find("frame").unwrap();
        let end = output.find("\x1b[?2026l").unwrap();
        assert!(begin < frame && frame < end);
    }

    #[test]
    fn synchronized_update_ends_marker_when_frame_fails() {
        let output = Rc::new(RefCell::new(Vec::new()));
        let writer = SharedWriter(Rc::clone(&output));
        let mut terminal = Terminal::new(CrosstermBackend::new(writer)).unwrap();
        let result = draw_synchronized_on(&mut terminal, |_| Err(io::Error::other("frame failed")));
        assert!(result.is_err());

        let output = String::from_utf8(output.borrow().clone()).unwrap();
        assert!(output.contains("\x1b[?2026l"));
    }

    #[test]
    fn leave_terminal_emits_alternate_screen_and_mouse_cleanup() {
        let mut output = Vec::new();
        leave_terminal(&mut output).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("\x1b[?1049l"));
        assert!(output.contains("\x1b[?1006l"));
        assert!(output.contains("\x1b[?1015l"));
        assert!(output.contains("\x1b[?1003l"));
    }

    #[test]
    fn leave_terminal_propagates_writer_errors() {
        struct BrokenWriter;

        impl Write for BrokenWriter {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::other("broken terminal"))
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        assert!(leave_terminal(&mut BrokenWriter).is_err());
    }
}
