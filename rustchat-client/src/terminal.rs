use chat_security::SessionCryptData;
use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    event::{self, Event, KeyCode},
    style::{self, Stylize},
    terminal::{self, ClearType, disable_raw_mode, enable_raw_mode},
};
use std::io::{self, Write};

use crate::message;

pub struct ChatWindow {
    messages: Vec<String>,
    input_buffer: String,
    window_height: u16,
}

impl ChatWindow {
    fn new() -> io::Result<Self> {
        let (_, height) = terminal::size()?;
        Ok(Self {
            messages: Vec::new(),
            input_buffer: String::new(),
            window_height: height,
        })
    }

    fn draw(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        // Clear screen
        stdout.execute(terminal::Clear(ClearType::All))?;

        // Draw messages
        let visible_messages = self
            .messages
            .iter()
            .rev()
            .take((self.window_height - 4) as usize)
            .rev();

        for (idx, message) in visible_messages.enumerate() {
            stdout
                .queue(cursor::MoveTo(1, (idx + 1) as u16))?
                .queue(style::PrintStyledContent(message.clone().white()))?;
        }

        // Draw input area
        stdout
            .queue(cursor::MoveTo(1, self.window_height - 2))?
            .queue(style::PrintStyledContent(
                format!("> {}", self.input_buffer).green(),
            ))?;

        stdout.flush()?;
        Ok(())
    }

    pub fn run_main(mut session: SessionCryptData, self_name: &str) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        let mut chat = ChatWindow::new()?;
        chat.messages.push(
            "Welcome to the chat! Type your messages below, or press Esc to quit".to_string(),
        );
        loop {
            chat.draw(&mut stdout)?;

            match session.check_data_available()? {
                true => {
                    let data = session.recieve_message()?;
                    chat.messages
                        .push(format!("{}> {}", data.sender_id, data.contents));
                }
                false => {
                    // No data available
                }
            }
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key_event) = event::read()? {
                    match key_event.code {
                        KeyCode::Enter => {
                            if !chat.input_buffer.is_empty() {
                                chat.messages.push(format!("{}> {}", self_name, chat.input_buffer.clone()));
                                session.send_message(message(&chat.input_buffer, self_name))?;
                                chat.input_buffer.clear();
                            }
                        }
                        KeyCode::Char(c) => {
                            chat.input_buffer.push(c);
                        }
                        KeyCode::Backspace => {
                            chat.input_buffer.pop();
                        }
                        KeyCode::Esc => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        disable_raw_mode()?;
        stdout.execute(terminal::Clear(ClearType::All))?;
        stdout.execute(cursor::MoveTo(0, 0))?;

        Ok(())
    }
}
