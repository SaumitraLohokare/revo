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
    event::read,
    execute,
    style::ResetColor,
    terminal::{disable_raw_mode, EnableLineWrap, LeaveAlternateScreen},
};
use editor::{Editor, EditorEvent};

mod buffer;
mod editor;
mod settings;
mod status_line;
mod terminal;
mod theme;
mod vec_ext;

fn main() -> io::Result<()> {
    setup_panic_handler();

    let settings = settings::read_editor_settings()?;

    // Create channel for editor events
    let (send, recv) = mpsc::channel();
    let input_send = send.clone();

    parse_args(send.clone())?;

    // Start input handeling thread
    let input_thread = std::thread::spawn(move || input(input_send));

    {
        let mut editor = Editor::new(settings, stdout(), send, recv)?;

        editor.start()?;
    }

    // NOTE: Editor needs to be dropped before we try to join input_thread
    input_thread
        .join()
        .expect("Failed while joining Input Thread");

    Ok(())
}

/// If a file or folder was passed as argument sends `EditorEvent::OpenFile` to the editor
fn parse_args(msg_sender: Sender<EditorEvent>) -> io::Result<()> {
    // Get the arguments passed to the program
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Ok(());
    } else if args.len() > 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Revo currently only supports opening one file at a time",
        ));
    }

    let path = PathBuf::from(&args[1]);

    if path.file_name().is_none() || (path.exists() && path.is_dir()) {
        // Opening a folder
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Opening directories is not yet supported",
        ))
    } else {
        // Opening a file
        msg_sender.send(EditorEvent::OpenFile(path)).unwrap();
        Ok(())
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

fn setup_panic_handler() {
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
}
