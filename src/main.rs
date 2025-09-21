#![windows_subsystem = "windows"]

use editor::editor::{Editor, Event as EditorEvent, Message as EditorMessage};
use iced::Element;
use iced::window;
use iced::window::icon;
use iced::{Task, Length};
use iced::widget::{column, container, scrollable, text};
use iced::widget::markdown::{self, Settings as MdSettings, Style as MdStyle};
use iced::Theme;
use notes::notes::{Event as NotesEvent, Message as NotesMessage, Note, Notes};
use settings::settings::{SettingsState, Message as SettingsMessage};
use std::collections::HashMap;

mod editor;
mod notes;
mod settings;


fn main() -> iced::Result {
    iced::daemon(|app: &App, id: window::Id| {
            match app.state.windows.get(&id).copied() {
                Some(WindowView::Note(index)) => {
                    let title = app.state.notes.get(index).map(|n| n.title.as_str()).unwrap_or("Note");
                    if title.trim().is_empty() {
                        String::from("faitout - Untitled page")
                    } else {
                        format!("faitout - {}", title)
                    }
                }
                _ => String::from("faitout"),
            }
        }, App::update, App::view)
        .theme(|app: &App, _id: window::Id| app.state.settings.theme())
        .subscription(|_app: &App| window::close_events().map(Message::WindowClosed))
        .run_with(|| {
            let mut app = App::default();
            let settings = window::Settings {
                icon: load_app_icon(),
                ..Default::default()
            };
            let (id, task) = window::open(settings);
            app.state.windows.insert(id, WindowView::Main);
            (app, task.map(Message::WindowOpened))
        })
}

#[derive(Default)]
struct App {
    state: State,
}

#[derive(Default)]
struct State {
    screen: Screen,
    editor: Editor,
    notes: Notes,
    settings: SettingsState,
    windows: HashMap<window::Id, WindowView>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum Screen {
    #[default]
    Notes,
    Editor,
    Settings
}

#[derive(Debug, Clone)]
enum Message {
    Editor(EditorMessage),
    Notes(NotesMessage),
    Settings(SettingsMessage),
    WindowOpened(window::Id),
    WindowClosed(window::Id),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowView {
    Main,
    Note(usize),
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Editor(message) => {
                if let Some(event) = self.state.editor.update(message) {
                    match event {
                        EditorEvent::Save {
                            title,
                            body,
                            tags,
                            editing,
                        } => {
                            let note = Note::new(title, body, tags);

                            let saved_index = self.state.notes.upsert(note, editing);
                            self.state.notes.select(Some(saved_index));
                            self.state.editor.load_new();
                            self.state.screen = Screen::Notes;
                        }
                        EditorEvent::Back => {
                            self.state.editor.load_new();
                            self.state.screen = Screen::Notes;
                        }
                    }
                }
                Task::none()
            }
            Message::Notes(message) => {
                if let Some(event) = self.state.notes.update(message) {
                    match event {
                        NotesEvent::Create => {
                            self.state.editor.load_new();
                            self.state.screen = Screen::Editor;
                            Task::none()
                        }
                        NotesEvent::Edit(index) => {
                            if let Some(note) = self.state.notes.get(index) {
                                self.state.editor.load_existing(index, note);
                                self.state.notes.select(Some(index));
                                self.state.screen = Screen::Editor;
                                Task::none()
                            }
                            else { Task::none() }
                        }
                        NotesEvent::Delete(index) => {
                            self.state.editor.adjust_after_delete(index);

                            if let Some(current) = self.state.editor.editing() {
                                self.state.notes.select(Some(current));
                            } else {
                                self.state.notes.select(None);
                                self.state.screen = Screen::Notes;
                            }
                            Task::none()
                        }
                        NotesEvent::OpenSettings => {
                            self.state.screen = Screen::Settings;
                            Task::none()
                        }
                        NotesEvent::OpenInNewWindow(index) => {
                            let settings = window::Settings {
                                icon: load_app_icon(),
                                ..Default::default()
                            };
                            let (id, task) = window::open(settings);
                            self.state.windows.insert(id, WindowView::Note(index));
                            task.map(Message::WindowOpened)
                        }
                    }
                } else { Task::none() }
            }

            Message::Settings(message) => {
                match message {
                    SettingsMessage::Back => {
                        self.state.screen = Screen::Notes;
                    }
                    _ => self.state.settings.update(message),
                }
                Task::none()
            }
            Message::WindowOpened(_id) => {
                // Window mapping already stored synchronously; nothing to do here.
                Task::none()
            }
            Message::WindowClosed(id) => {
                self.state.windows.remove(&id);
                if self.state.windows.is_empty() {
                    iced::exit()
                } else {
                    Task::none()
                }
            }
        }
    }

    fn view(&self, id: window::Id) -> Element<'_, Message> {
        match self.state.windows.get(&id).copied() {
            Some(WindowView::Main) | None => match self.state.screen {
                Screen::Editor => self.state.editor.view().map(Message::Editor),
                Screen::Notes => self.state.notes.view().map(Message::Notes),
                Screen::Settings => self.state.settings.view().map(Message::Settings),
            },
            Some(WindowView::Note(index)) => self.note_window_view(index),
        }
    }

    fn note_window_view(&self, index: usize) -> Element<'_, Message> {
        if let Some(note) = self.state.notes.get(index) {
            let title = if note.title.trim().is_empty() { "Untitled page" } else { &note.title };

            let md_style = MdStyle::from_palette(Theme::KanagawaDragon.palette());
            let preview = markdown::view(note.parsed(), MdSettings::default(), md_style)
                .map(|_| Message::Notes(NotesMessage::LinkClicked));

            let content = column![
                text(title).size(26),
                scrollable(preview).height(Length::Fill),
            ]
            .spacing(12)
            .padding(16);

            container(content).width(Length::Fill).height(Length::Fill).into()
        } else {
            container(text("Note not found").size(16)).padding(24).into()
        }
    }
}

fn load_app_icon() -> Option<window::Icon> {
    // Prefer .ico on Windows, fallback to .png if available
    // Paths are relative to the current working directory
    let ico = icon::from_file("assets/icon.ico");
    match ico {
        Ok(icon) => Some(icon),
        Err(_) => icon::from_file("assets/icon.png").ok(),
    }
}

