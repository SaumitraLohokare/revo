use std::{
    env,
    io::{self, stdout},
    panic,
    path::PathBuf,
    process::exit,
    sync::mpsc::{self, Sender},
};

use buffer::{Buffer, BufferLogic};
use crossterm::{
    cursor::{SetCursorStyle, Show},
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::ResetColor,
    terminal::{disable_raw_mode, EnableLineWrap, LeaveAlternateScreen},
};
use editor::{Editor, EditorEvent};

mod buffer;
mod editor;
mod settings;
mod status_line;
mod string;
mod terminal;
mod theme;
mod vec_ext;

fn main() {
    panic::set_hook(Box::new(|panic_info| {
        if let Err(e) = disable_raw_mode() {
            eprintln!("ERROR : Failed to disable terminal raw mode : {e}");
            exit(1);
        }

        if let Err(e) = execute!(
            stdout(),
            ResetColor,
            LeaveAlternateScreen,
            EnableLineWrap,
            SetCursorStyle::BlinkingBlock,
            Show
        ) {
            eprintln!("ERROR : Failed to leave alternate screen : {e}");
            exit(1);
        }

        if let Some(location) = panic_info.location() {
            println!("Panic occurred at: {}:{}", location.file(), location.line());
        }
        println!("Panic payload: {:?}", panic_info.payload());
    }));

    if let Err(e) = run() {
        eprintln!("ERROR : {e}");
        exit(1);
    }
}

fn run() -> io::Result<()> {
    let settings = settings::read_editor_settings()?;

    let stdout = io::stdout();

    let (send, recv) = mpsc::channel();
    let input_send = send.clone();
    let input_thread = std::thread::spawn(move || input(input_send));

    let mut editor = Editor::new(settings, stdout, send.clone())?;

    match parse_args(editor.terminal.width, editor.terminal.height - 1, send)? {
        Some(buf) => {
            let id = editor.add_buffer(buf);
            editor.activate_buffer(id);
        }
        None => editor.terminal.draw_welcome_msg(),
    }

    editor.update_status_line_file();

    loop {
        if let Ok(event) = recv.recv() {
            match event {
                EditorEvent::Input(event) => match event {
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    }) => break,

                    Event::Resize(w, h) => editor.resize(w, h),

                    _ => editor.forward_event(event),
                },
                EditorEvent::Buffer(buffer_event) => editor.handle_buffer_event(buffer_event)?,
            }
        } else {
            unreachable!("As long as input thread is running, this should never be reached.");
        }

        editor.begin_draw()?;

        editor.draw_buffers();

        editor.update_status_line_cursor();
        editor.draw_status_line();

        editor.end_draw()?;
        editor.show_cursor()?;
    }

    drop(recv);

    input_thread
        .join()
        .expect("Failed while joining Input Thread");

    Ok(())
}

fn parse_args(
    width: u16,
    height: u16,
    msg_sender: Sender<EditorEvent>,
) -> io::Result<Option<Buffer>> {
    // Get the arguments passed to the program
    let args: Vec<String> = env::args().collect();

    // If no arguments were passed (other than the program name), return None
    if args.len() < 2 {
        return Ok(None);
    }

    // The second argument (args[1]) is expected to be the file path
    let path = PathBuf::from(&args[1]);

    if path.file_name().is_none() || (path.exists() && path.is_dir()) {
        // Opening a folder
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Opening directories is not yet supported",
        ));
    } else {
        // Opening a file
        // Attempt to create a new Buffer with the provided file path
        // The x, y, width, and height values are set to default values (you can adjust these as needed)
        let x = 0;
        let y = 0;

        // Return the Buffer inside an Option, or None if there was an error
        Buffer::new(path, x, y, width, height, false, BufferLogic::Editor, "", msg_sender).map(|b| Some(b))
    }
}

fn input(out: Sender<EditorEvent>) {
    loop {
        if let Ok(event) = read() {
            if let Err(_) = out.send(EditorEvent::Input(event)) {
                break;
            }
        }
    }
}
