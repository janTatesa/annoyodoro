use crate::App;
use cli_log::debug;
use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::DefaultTerminal;

impl App {
    pub fn handle_event(&mut self, event: Event, terminal: &mut DefaultTerminal) -> Result<()> {
        debug!("{:?}", event);
        if let Event::Key(KeyEvent {
            code: KeyCode::Char(key),
            modifiers: KeyModifiers::NONE,
            ..
        }) = event
        {
            self.handle_key_event(key)?;
        }

        terminal.draw(|frame| self.render(frame))?;

        Ok(())
    }

    fn handle_key_event(&mut self, key: char) -> Result<()> {
        match key {
            'q' => self.running = false,
            'e' => self.break_time(true)?,
            'p' => self.toggle_pause(),
            _ => {}
        }
        Ok(())
    }

    fn toggle_pause(&mut self) {
        self.is_paused = match self.is_paused {
            true => {
                self.ticker.reset();
                false
            }
            false => true,
        }
    }
}
