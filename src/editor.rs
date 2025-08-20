mod command_bar;
mod document_status;
mod editor_cmd;
mod line;
mod message_bar;
mod status_bar;
mod terminal;
mod ui_component;
mod view;

use std::{fmt::Display, io::ErrorKind, time::Duration};

use crossterm::event::{Event, KeyEvent, KeyEventKind, read};
use editor_cmd::{EditorCommand, TextCommand};
use terminal::Terminal;
use view::View;

use crate::editor::{
    command_bar::{Cmd, CommandBar},
    message_bar::MessageBar,
    status_bar::StatusBar,
    terminal::{Position, TerminalSize},
    ui_component::UiComponent,
};

const TIMES_TO_QUIT: u8 = 3;
const MESSAGE_DURATION: Duration = Duration::new(5, 0);
const DEFAULT_MESSAGE: &str = "HELP: '/' = find | Ctrl-S = save | Ctrl-Q = quit";

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum EditorMode {
    #[default]
    Normal,
    Insert,
    Command,
}

impl Display for EditorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                EditorMode::Normal => "NORMAL",
                EditorMode::Insert => "INSERT",
                EditorMode::Command => "COMMAND",
            }
        )
    }
}

#[derive(Default)]
pub struct Editor {
    mode: EditorMode,
    switched_mode: bool,
    should_quit: bool,
    view: View,
    status_bar: StatusBar,
    message_bar: MessageBar,
    command_bar: CommandBar,
    size: TerminalSize,
    pressed_quit: u8,
}

impl Editor {
    /// Creates a new instance of the text editor
    /// and sets a panic hook for terminating correcly
    /// even when unwinding during panic.
    pub fn new() -> Result<Self, std::io::Error> {
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            default_hook(panic_info);
        }));

        Terminal::initialize()?;
        let mut editor = Editor::default();

        let args: Vec<String> = std::env::args().collect();
        let file_name = args.get(1);
        let mut init_message = DEFAULT_MESSAGE.to_string();
        if let Some(path) = file_name {
            let res = editor.view.load(path);
            match res {
                Ok(()) => Terminal::set_title(path)?,
                Err(_) => init_message = format!("ERR: Could not open file: {path}"),
            }
            Terminal::set_title(path)?;
        }

        let size = Terminal::size().unwrap_or_default();

        editor.resize(size);
        editor.message_bar.set_message(&init_message);
        let status = editor.view.get_status();
        editor.status_bar.update_status(status);

        editor.pressed_quit = TIMES_TO_QUIT;
        Ok(editor)
    }

    fn resize(&mut self, size: TerminalSize) {
        self.size = size;

        self.view.resize(TerminalSize {
            height: size.height.saturating_sub(2),
            width: size.width,
        });

        self.message_bar.resize(TerminalSize {
            height: 1,
            width: size.width,
        });

        self.status_bar.resize(TerminalSize {
            height: 1,
            width: size.width,
        });

        self.command_bar.resize(TerminalSize {
            height: 1,
            width: size.width,
        });
    }

    /// Runs the editor with a infinite loop that reads
    /// every event from keyboard, evaluates it and refreshes
    /// the screen.
    pub fn run(&mut self) {
        loop {
            self.refresh_screen();

            if self.should_quit {
                break;
            }

            let event = read();
            match event {
                Ok(event) => self.evaluate_event(event),
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    panic!("Unrecognized event, error: {_err:?}");
                }
            }

            let status = self.view.get_status();
            self.status_bar.update_status(status);
            self.status_bar.update_editor_mode(self.mode);
        }
    }

    /// Evaluates an event from the keyboard and resizing
    fn evaluate_event(&mut self, event: Event) {
        let should_process = match event {
            Event::Key(KeyEvent { kind, .. }) => kind == KeyEventKind::Press,
            Event::Resize(_, _) => true,
            _ => false,
        };

        if should_process {
            match self.mode {
                EditorMode::Normal => {
                    if let Ok(cmd) = EditorCommand::try_from(event) {
                        self.process_normal_command(cmd);
                    }
                }
                EditorMode::Insert => {
                    if let Ok(cmd) = TextCommand::try_from(event) {
                        self.process_insertion(cmd);
                    }
                }
                EditorMode::Command => {
                    if let Ok(cmd) = TextCommand::try_from(event) {
                        self.process_command(cmd);
                    }
                }
            }
        } else {
            #[cfg(debug_assertions)]
            panic!("Press Event could not be processed\n");
        }
    }

    fn enter_command_mode(&mut self, cmd: Cmd) {
        self.mode = EditorMode::Command;
        self.command_bar.set_command(cmd);
        self.switched_mode = true;
    }

    fn exit_command_mode(&mut self) {
        self.command_bar.clear();
        self.mode = EditorMode::Normal;
        self.switched_mode = true;
    }

    fn execute_command(&mut self) {
        let cmd = self.command_bar.get_command().expect("Command wasn't set");
        match cmd {
            Cmd::Search => {
                let needle = self.command_bar.get_line();
                self.view.set_search_term(needle);
                self.view.move_to_first_occurrence();
            }
            Cmd::SaveAs => {
                let file_name = self.command_bar.get_line();
                let _ = self.view.save_as(&file_name);
                self.message_bar.set_message("File was saved successfully");
            }
        }
    }

    fn search_next(&mut self) {
        self.view.move_to_next_occurrence();
    }

    fn search_prev(&mut self) {
        self.view.move_to_prev_occurrence();
    }

    fn process_command(&mut self, cmd: TextCommand) {
        match cmd {
            TextCommand::Write(symbol) => self.command_bar.handle_insertion(symbol),
            TextCommand::Deletion => self.command_bar.handle_deletion(),
            TextCommand::Backspace => self.command_bar.handle_backspace(),
            TextCommand::Exit => self.exit_command_mode(),
            TextCommand::Enter => {
                self.execute_command();
                self.exit_command_mode();
            }
        }
    }

    fn process_insertion(&mut self, cmd: TextCommand) {
        match cmd {
            TextCommand::Write(symbol) => self.view.handle_insertion(symbol),
            TextCommand::Enter => self.view.handle_enter(),
            TextCommand::Deletion => self.view.handle_deletion(),
            TextCommand::Backspace => self.view.handle_backspace(),
            TextCommand::Exit => {
                self.mode = EditorMode::Normal;
                self.switched_mode = true;
            }
        }
    }

    fn warn_unsaved_file(&mut self) {
        if self.pressed_quit.checked_sub(1).is_none() {
            self.should_quit = true;
        } else {
            self.message_bar.set_message(&format!(
                "WARNING! File has unsaved changes. Press Ctrl-Q {times} more times to quit.",
                times = self.pressed_quit
            ));
            self.pressed_quit = self.pressed_quit.saturating_sub(1);
        }
    }

    fn process_normal_command(&mut self, cmd: EditorCommand) {
        match cmd {
            EditorCommand::Search => self.enter_command_mode(Cmd::Search),
            EditorCommand::NextOccurrence => self.search_next(),
            EditorCommand::PrevOccurrence => self.search_prev(),
            EditorCommand::Save => {
                let res = self.view.save();
                match res {
                    Ok(()) => {
                        self.pressed_quit = TIMES_TO_QUIT;
                        self.message_bar.set_message("File was saved successfully");
                    }
                    Err(err) if err.kind() == ErrorKind::NotFound => {
                        self.enter_command_mode(Cmd::SaveAs);
                    }
                    Err(_) => self.message_bar.set_message("Error writing file"),
                }
            }

            EditorCommand::Quit => {
                if self.view.is_file_modified() {
                    self.warn_unsaved_file();
                } else {
                    self.should_quit = true;
                }
            }

            EditorCommand::EnterInsert => {
                self.mode = EditorMode::Insert;
                self.switched_mode = true;
            }
            _ => self.view.handle_command(cmd),
        }

        if let EditorCommand::Resize(size) = cmd {
            self.status_bar.resize(size);
        }
    }

    /// Refreshes the screen in order to render correcly the events
    fn refresh_screen(&mut self) {
        if self.size.width == 0 || self.size.height == 0 {
            return;
        }

        let _ = Terminal::hide_cursor();

        if self.switched_mode {
            let _ = match self.mode {
                EditorMode::Normal => Terminal::cursor_block(),
                EditorMode::Command | EditorMode::Insert => Terminal::cursor_bar(),
            };
            self.switched_mode = false;
        }

        let mut cursor_pos = self.view.cursor_position();

        if let EditorMode::Command = self.mode {
            let y = self.size.height.saturating_sub(1);
            cursor_pos = Position {
                x: self.command_bar.cursor_location(),
                y,
            };
            self.command_bar.render(y);
            self.message_bar.set_needs_redraw(true);
        } else {
            self.message_bar.render(self.size.height.saturating_sub(1));
        }

        if self.size.height > 1 {
            self.status_bar.render(self.size.height.saturating_sub(2));
        }

        if self.size.height > 2 {
            self.view.render(0);
        }

        let _ = Terminal::move_cursor_to(cursor_pos);
        let _ = Terminal::show_cursor();
        let _ = Terminal::execute();
    }
}

impl Drop for Editor {
    /// Destructor of the editor for terminating correcly when the
    /// program finishes. Since it can possibly panic a panic hook is
    /// also implemented.
    fn drop(&mut self) {
        let _ = Terminal::terminate();
        let _ = Terminal::cursor_block();
        if self.should_quit {
            Terminal::print("Goodbye.\r\n").unwrap();
        }
    }
}
