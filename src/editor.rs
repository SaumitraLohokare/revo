#![allow(dead_code)]
use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
    path::PathBuf,
    sync::mpsc::{Receiver, Sender},
};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use uuid::Uuid;

use crate::{
    buffer::{Buffer, BufferLogic},
    settings::Settings,
    terminal::Terminal,
};

pub enum BufferEvent {
    Save {
        id: Uuid,
    },
    SaveAs {
        id: Uuid,
    },
    Close {
        id: Uuid,
        is_overlay: bool,
    },
    ResumeEvent {
        paused_event_id: Uuid,
        result: String,
    },
    CancelEvent {
        paused_event_id: Uuid,
    },
}

// TODO:
// FocusEvent (Think about how to implement this)
// ResizeBuffers (Maybe make this an event)
// OpenFile?
// OpenFileInSplit?
// OpenFolder?
//
// Can maybe add ReloadSettings to support hot-reloading
pub enum EditorEvent {
    Input(Event),
    Buffer(BufferEvent),
    OpenFile(PathBuf),
}

pub struct PausedEvent {
    id: Uuid,
    event: EditorEvent,
}

// TODO: Might wanna change active_buffer, active_overlays to a stack of focus events
// 		 this can make it easier to keep rewinding the focus
pub struct Editor<W: Write> {
    settings: Settings,

    buffers: HashMap<Uuid, Buffer>,
    active_buffer: Option<Uuid>,

    overlays: HashMap<Uuid, Buffer>,
    active_overlay: Option<Uuid>,

    pub terminal: Terminal<W>,

    msg_receiver: Receiver<EditorEvent>,
    sender_copy: Sender<EditorEvent>,

    paused_events: Vec<PausedEvent>,
}

impl<W: Write> Editor<W> {
    pub fn new(
        settings: Settings,
        out: W,
        sender_copy: Sender<EditorEvent>,
        msg_receiver: Receiver<EditorEvent>,
    ) -> io::Result<Self> {
        Ok(Self {
            settings,
            buffers: HashMap::new(),
            active_buffer: None,
            overlays: HashMap::new(),
            active_overlay: None,
            terminal: Terminal::new(out)?,
            msg_receiver,
            sender_copy,
            paused_events: vec![],
        })
    }

    pub fn start(&mut self) -> io::Result<()> {
        loop {
            // TODO: Maybe make it so that we onlly draw when correct events are triggered
            // 		 Drawing on ALL events is a waste (such as Save...)
            if let Ok(event) = self.msg_receiver.recv() {
                match event {
                    EditorEvent::Input(event) => match event {
                        Event::Key(KeyEvent {
                            code: KeyCode::Char('q'),
                            modifiers: KeyModifiers::CONTROL,
                            ..
                        }) => break,

                        Event::Resize(w, h) => self.resize(w, h),

                        _ => self.forward_event(event),
                    },
                    EditorEvent::Buffer(buffer_event) => self.handle_buffer_event(buffer_event)?,
                    EditorEvent::OpenFile(path_buf) => {
                        // The x, y, width, and height values are set to default values (you can adjust these as needed)
                        let x = 0;
                        let y = 0;

                        // Return the Buffer inside an Option, or None if there was an error
                        let buf = Buffer::new(
                            path_buf,
                            x,
                            y,
                            self.terminal.width,
                            self.terminal.height,
                            true,
                            false,
                            BufferLogic::Editor,
                            "",
                            self.sender_copy.clone(),
                        )?;

                        let id = self.add_buffer(buf);
                        self.activate_buffer(id);
                    }
                }
            } else {
                unreachable!("As long as input thread is running, this should never be reached.");
            }

            self.begin_draw()?;

            self.draw_buffers();

            self.end_draw()?;
            self.show_cursor()?;
        }

        Ok(())
    }

    pub fn resize(&mut self, w: u16, h: u16) {
        // TODO: Handling resize of active buffers?

        self.terminal.resize(w, h);
    }

    pub fn add_buffer(&mut self, mut buf: Buffer) -> Uuid {
        let uuid = Uuid::new_v4();
        buf.id = uuid;
        buf.is_overlay = false;
        let result = self.buffers.insert(buf.id, buf);
        assert!(result.is_none()); // If this goes through means we accidentally wrote over a previous buffer
        uuid
    }

    pub fn activate_buffer(&mut self, id: Uuid) {
        if self.buffers.contains_key(&id) {
            self.active_buffer = Some(id);
        } else {
            panic!("Tried to activate an invalid buffer.");
        }
    }

    pub fn add_overlay(&mut self, mut ov: Buffer) -> Uuid {
        let id = Uuid::new_v4();
        ov.id = id;
        ov.is_overlay = true;
        let result = self.overlays.insert(ov.id, ov);
        assert!(result.is_none()); // If this goes through means we accidentally wrote over a previous buffer
        id
    }

    pub fn activate_overlay(&mut self, id: Uuid) {
        if self.overlays.contains_key(&id) {
            self.active_overlay = Some(id);
        } else {
            panic!("Tried to activate an invalid buffer.");
        }
    }

    pub fn close_overlay(&mut self, id: Uuid) {
        if self.overlays.contains_key(&id) {
            if self.active_overlay == Some(id) {
                self.active_overlay = None;
            }

            self.overlays.remove(&id);
        } else {
            panic!("Tried to remove an invalid buffer.");
        }
    }

    pub fn begin_draw(&mut self) -> io::Result<()> {
        self.terminal.begin_draw(&self.settings.theme)
    }

    pub fn end_draw(&mut self) -> io::Result<()> {
        self.terminal.end_draw()
    }

    pub fn draw_buffers(&mut self) {
        if self.buffers.len() > 0 {
            for buf in self.buffers.values().filter(|b| b.visible) {
                self.terminal.draw_buffer(buf, &self.settings.theme);
            }
        } else {
            self.terminal.draw_welcome_msg();
        }

        if self.overlays.len() > 0 {
            for buf in self.overlays.values().filter(|b| b.visible) {
                self.terminal.draw_buffer(buf, &self.settings.theme);
            }
        }
    }

    // TODO: This will work with FocusStack
    pub fn show_cursor(&mut self) -> io::Result<()> {
        if let Some(id) = self.active_overlay {
            self.terminal.show_cursor(self.overlays.get(&id).unwrap())?;
        } else if let Some(id) = self.active_buffer {
            self.terminal.show_cursor(self.buffers.get(&id).unwrap())?;
        }

        Ok(())
    }

    // TODO: This will work with FocusStack
    pub fn forward_event(&mut self, event: Event) {
        if let Some(id) = self.active_overlay {
            self.overlays.get_mut(&id).unwrap().parse_input(event);
        } else if let Some(id) = self.active_buffer {
            self.buffers.get_mut(&id).unwrap().parse_input(event);
        }
    }

    pub fn handle_buffer_event(&mut self, event: BufferEvent) -> io::Result<()> {
        match event {
            BufferEvent::Save { id } => {
                self.save_buffer(id)?;
            }
            BufferEvent::SaveAs { .. } => {
                // Pause this event
                let paused_event_id = Uuid::new_v4();
                self.paused_events.push(PausedEvent {
                    id: paused_event_id,
                    event: EditorEvent::Buffer(event),
                });

                // Open up a new overlay
                let width = 32;
                let height = 3;
                let x = (self.terminal.width / 2).saturating_sub(width / 2);
                let y = (self.terminal.height / 2).saturating_sub(height);

                let mut overlay = Buffer::new(
                    PathBuf::from("Save As:"),
                    x,
                    y,
                    width,
                    height,
                    false,
                    true,
                    BufferLogic::InputBox,
                    "Save As",
                    self.sender_copy.clone(),
                )?;
                overlay.set_paused_event_id(paused_event_id);
                let id = self.add_overlay(overlay);

                // Activate that overlay
                self.activate_overlay(id);
            }
            BufferEvent::Close { id, is_overlay } => {
                if is_overlay {
                    self.close_overlay(id);
                } else {
                    todo!("Add closing for normal buffers");
                }
            }
            BufferEvent::ResumeEvent {
                paused_event_id,
                result,
            } => {
                // Look through our paused events and match this id
                if let Some(event) = self
                    .paused_events
                    .iter()
                    .filter(|e| e.id == paused_event_id)
                    .next()
                {
                    match event.event {
                        EditorEvent::Buffer(BufferEvent::SaveAs { id }) => {
                            self.save_buffer_as(id, result)?;
                        }
                        _ => (),
                    }
                }
            }
            BufferEvent::CancelEvent { paused_event_id } => {
                // Look through our paused events and match this id
                if let Some((i, _)) = self
                    .paused_events
                    .iter()
                    .filter(|e| e.id == paused_event_id)
                    .enumerate()
                    .next()
                {
                    self.paused_events.remove(i);
                }
            }
        }

        Ok(())
    }

    // TODO: Think about moving this to a separate thread
    fn save_buffer(&mut self, id: Uuid) -> io::Result<()> {
        if let Some(buf) = self.buffers.get(&id) {
            let contents: String = buf.data.to_string();
            if let Some(file_path) = &buf.file_path {
                fs::write(file_path, contents)?;
            } else {
                // If we try to save a buffer without a name...
                self.handle_buffer_event(BufferEvent::SaveAs { id })?;
            }
        }

        Ok(())
    }

    // TODO: Think about moving this to a separate thread
    fn save_buffer_as(&mut self, id: Uuid, file_name: String) -> io::Result<()> {
        if let Some(buf) = self.buffers.get_mut(&id) {
            let contents: String = buf.data.to_string();

            match &mut buf.file_path {
                Some(path) => path.set_file_name(file_name),
                None => buf.set_path(PathBuf::from(file_name)).unwrap(), // TODO: Ensure correct handling of failed attempt to save as
            }

            if let Some(file_path) = &buf.file_path {
                fs::write(file_path, contents)?;
            }
        }

        Ok(())
    }
}
