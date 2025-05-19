mod editor_cmd;
mod terminal;
mod view;

use crossterm::event::{Event, KeyEvent, KeyEventKind, read};
use editor_cmd::{EditorCommand, EditorCommandInsert};
use terminal::Terminal;
use view::View;

#[derive(Clone, Copy)]
pub enum EditorMode {
    Normal,
    Insert,
}

pub struct Editor {
    mode: EditorMode,
    switched_mode: bool,
    should_quit: bool,
    view: View,
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

        let mut view = View::new();
        let args: Vec<String> = std::env::args().collect();
        if let Some(path) = args.get(1) {
            view.load(path);
        }

        Ok(Self {
            mode: EditorMode::Normal,
            should_quit: false,
            switched_mode: false,
            view,
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
