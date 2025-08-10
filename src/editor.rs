mod editor_cmd;
mod status_bar;
mod terminal;
mod view;

use std::fmt::{Debug, Display};

use crossterm::event::{Event, KeyEvent, KeyEventKind, read};
use editor_cmd::{EditorCommand, EditorCommandInsert};
use terminal::Terminal;
use view::View;

use crate::editor::status_bar::StatusBar;

const BARS_COUNT: usize = 2;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum EditorMode {
    #[default]
    Normal,
    Insert,
}

impl Display for EditorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                EditorMode::Normal => "NORMAL",
                EditorMode::Insert => "INSERT",
            }
        )
    }
}

pub struct Editor {
    mode: EditorMode,
    switched_mode: bool,
    should_quit: bool,
    view: View,
    status_bar: StatusBar,
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct DocumentStatus {
    file_name: Option<String>,
    num_of_lines: usize,
    current_line: usize,
    modified: bool,
}

impl Debug for DocumentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.file_name.clone().unwrap_or("[No Name]".to_string()),
            self.num_of_lines,
            self.current_line,
            self.modified
        )
    }
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
        let mut view = View::new(BARS_COUNT);

        let args: Vec<String> = std::env::args().collect();
        let file_name = args.get(1);
        if let Some(path) = file_name {
            view.load(path);
        }

        Ok(Self {
            mode: EditorMode::Normal,
            should_quit: false,
            switched_mode: false,
            view,
            status_bar: StatusBar::new(BARS_COUNT),
        })
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
                Err(err) => {
                    #[cfg(debug_assertions)]
                    panic!("Unrecognized event, error: {err:?}");
                }
            }
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
                        self.process_command(cmd);
                    }
                }
                EditorMode::Insert => {
                    if let Ok(cmd) = EditorCommandInsert::try_from(event) {
                        self.process_insertion(cmd);
                    }
                }
            }
        } else {
            #[cfg(debug_assertions)]
            panic!("Press Event could not be processed\n");
        }
    }

    fn process_insertion(&mut self, cmd: EditorCommandInsert) {
        match cmd {
            EditorCommandInsert::Write(symbol) => self.view.handle_insertion(symbol),
            EditorCommandInsert::Enter => self.view.handle_enter(),
            EditorCommandInsert::Deletion => self.view.handle_deletion(),
            EditorCommandInsert::Backspace => self.view.handle_backspace(),
            EditorCommandInsert::ExitInsert => {
                self.mode = EditorMode::Normal;
                self.switched_mode = true;
            }
        }
    }

    fn process_command(&mut self, cmd: EditorCommand) {
        match cmd {
            EditorCommand::Save => self.view.save(),
            EditorCommand::Quit => self.should_quit = true,
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
        let _ = Terminal::hide_cursor();

        if self.switched_mode {
            let _ = match self.mode {
                EditorMode::Normal => Terminal::cursor_block(),
                EditorMode::Insert => Terminal::cursor_bar(),
            };
            self.switched_mode = false;
        }

        self.view.render();
        self.status_bar.update_status(self.view.get_status());
        self.status_bar.update_editor_mode(self.mode);
        self.status_bar.render();

        let _ = Terminal::move_cursor_to(self.view.cursor_position());
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
