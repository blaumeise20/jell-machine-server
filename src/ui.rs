use std::{io::{stdout, Stdout}, process};

use ansi_cut::AnsiCut;
use async_channel::{Sender, Receiver};
use crossterm::{terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode, Clear, ClearType::CurrentLine, self, LeaveAlternateScreen}, execute, event::{EventStream, Event, KeyCode, KeyModifiers}, cursor, style::Print};
use futures::{StreamExt, FutureExt, stream::select, future};

use crate::{log::format_log, chat::ChatMessage};

pub fn create_ui() -> (Sender<String>, Receiver<ChatMessage>) {
    let (ls, lr) = async_channel::bounded(10);
    let (is, ir) = async_channel::bounded(20);
    let (cs, cr) = async_channel::unbounded();

    // drawing
    tokio::spawn(async move {
        enable_raw_mode().unwrap();
        let mut screen = Screen::new();
        screen.draw_log_window();
        screen.draw_input_bar();

        select(
            lr.map(ConsoleEvent::Log),
            ir.map(ConsoleEvent::UserEvent)
        ).for_each(|msg| {
            match msg {
                ConsoleEvent::Log(msg) => {
                    screen.log(msg);
                },
                ConsoleEvent::UserEvent(event) => {
                    let mut message = None;
                    screen.handle_user_event(event, &mut message);
                    if let Some(message) = message {
                        cs.try_send(ChatMessage { content: message, sender: "server".into() }).unwrap();
                    }
                },
            }
            future::ready(())
        }).await
    });

    // input handling
    tokio::spawn(async move {
        let mut reader = EventStream::new();
        while let Some(event) = reader.next().fuse().await {
            is.send(event.unwrap()).await.unwrap();
        }
    });

    (ls, cr)
}

enum ConsoleEvent {
    Log(String),
    UserEvent(Event),
}


struct Screen {
    logs: Vec<String>,

    current_input: String,
    position: usize,
    input_offset: usize,

    window_size: (u16, u16),
    stdout: Stdout,
}

impl Screen {
    fn new() -> Self {
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen).unwrap();

        Self {
            logs: vec![],

            current_input: String::new(),
            position: 0,
            input_offset: 0,

            window_size: terminal::size().unwrap(),
            stdout,
        }
    }

    fn log(&mut self, msg: String) {
        self.logs.push(msg);
        self.draw_log_window();
    }

    fn draw_log_window(&mut self) {
        let mut stdout = self.stdout.lock();

        let mut y = self.window_size.1 - 3; // two line for input
        for msg in self.logs.iter().rev() {
            execute!(
                stdout,
                cursor::MoveTo(0, y),
                Clear(CurrentLine),
                Print(msg.clone())
            ).unwrap();
            y -= 1;
            if y == 0 {
                break;
            }
        }

        execute!(stdout, cursor::MoveTo((self.position - self.input_offset + 2) as u16, self.window_size.1 - 1)).unwrap();
    }

    fn draw_input_bar(&mut self) {
        let width = self.window_size.0 as usize - 2;
        let start = self.input_offset;
        let end = (start + width).min(self.current_input.len());

        let highlighted = input_highlight(&self.current_input[start..end]);
        let drawn_input = format!("{:width$}", highlighted.cut(start..end), width=width);

        execute!(
            self.stdout,
            cursor::Hide,
            cursor::MoveTo(0, self.window_size.1 - 2),
            Clear(CurrentLine),
            Print("─".repeat(self.window_size.0 as usize)),
            cursor::MoveTo(0, self.window_size.1 - 1),
            Clear(CurrentLine),
            Print("> "),
            Print(drawn_input),
            cursor::MoveTo((self.position - self.input_offset + 2) as u16, self.window_size.1 - 1),
            cursor::Show
        ).unwrap();
    }

    fn handle_user_event(&mut self, event: Event, message: &mut Option<String>) {
        match event {
            Event::Key(key) => {
                let modifiers = key.modifiers;
                let key = key.code;
                match key {
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                        execute!(self.stdout, LeaveAlternateScreen).unwrap();
                        disable_raw_mode().unwrap();
                        process::exit(0);
                    }

                    KeyCode::Left => {
                        if self.position > 0 {
                            self.position -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if self.position < self.current_input.len() {
                            self.position += 1;
                        }
                    }

                    KeyCode::Backspace => {
                        if self.position > 0 {
                            self.current_input.remove(self.position - 1);
                            self.position -= 1;
                        }
                    }
                    KeyCode::Delete => {
                        if self.position < self.current_input.len() {
                            self.current_input.remove(self.position);
                        }
                    }

                    KeyCode::Home => {
                        self.position = 0;
                        self.input_offset = 0;
                    }
                    KeyCode::Char('a') if modifiers.contains(KeyModifiers::CONTROL) => {
                        self.position = 0;
                        self.input_offset = 0;
                    }
                    KeyCode::End => {
                        self.position = self.current_input.len();
                    }
                    KeyCode::Char('e') if modifiers.contains(KeyModifiers::CONTROL) => {
                        self.position = self.current_input.len();
                    }

                    KeyCode::Char('u') if modifiers.contains(KeyModifiers::CONTROL) => {
                        self.current_input = self.current_input[self.position..].to_string();
                        self.position = 0;
                        self.input_offset = 0;
                    }

                    KeyCode::Enter => {
                        if self.current_input.starts_with('/') {
                            self.log(format_log("94", "COMMAND", &self.current_input));
                        }
                        *message = Some(self.current_input.clone());


                        self.current_input.clear();
                        self.position = 0;
                        self.input_offset = 0;
                    }

                    KeyCode::Char(ch) => {
                        self.current_input.insert(self.position, ch);
                        self.position += 1;
                    }

                    _ => {},
                }
                self.draw_input_bar();
            }
            Event::Resize(w, h) => {
                self.window_size = (w, h);
                self.draw_log_window();
                self.draw_input_bar();
            }
            _ => {}
        }
    }
}

fn input_highlight(input: &str) -> String {
    let input = input.to_string();
    if input.starts_with('/') {
        let mut parsed = input.split(' ');
        let mut output = String::with_capacity(input.len());

        output.push_str("\x1b[1;36m");
        output.push_str(parsed.next().unwrap());
        output.push_str("\x1b[0m ");

        for part in parsed {
            if part.parse::<i32>().is_ok() {
                output.push_str("\x1b[1;35m");
            }
            else {
                output.push_str("\x1b[33m");
            }
            output.push_str(part);
            output.push_str("\x1b[0m ");
        }
        output
    }
    else {
        input
    }
}
