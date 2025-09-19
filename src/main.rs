use editor::editor::{Editor, Event as EditorEvent, Message as EditorMessage};
use iced::{Element, Theme};
use notes::notes::{Event as NotesEvent, Message as NotesMessage, Note, Notes};

mod editor;
mod notes;

fn main() -> iced::Result {
    iced::application("faitout", App::update, App::view)
        .theme(|_| Theme::KanagawaDragon)
        .run()
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
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum Screen {
    #[default]
    Notes,
    Editor,
}

#[derive(Debug, Clone)]
enum Message {
    Editor(EditorMessage),
    Notes(NotesMessage),
}

impl App {
    fn update(&mut self, message: Message) {
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
            }
            Message::Notes(message) => {
                if let Some(event) = self.state.notes.update(message) {
                    match event {
                        NotesEvent::Create => {
                            self.state.editor.load_new();
                            self.state.screen = Screen::Editor;
                        }
                        NotesEvent::Edit(index) => {
                            if let Some(note) = self.state.notes.get(index) {
                                self.state.editor.load_existing(index, note);
                                self.state.notes.select(Some(index));
                                self.state.screen = Screen::Editor;
                            }
                        }
                        NotesEvent::Delete(index) => {
                            self.state.editor.adjust_after_delete(index);

                            if let Some(current) = self.state.editor.editing() {
                                self.state.notes.select(Some(current));
                            } else {
                                self.state.notes.select(None);
                                self.state.screen = Screen::Notes;
                            }
                        }
                    }
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match self.state.screen {
            Screen::Editor => self.state.editor.view().map(Message::Editor),
            Screen::Notes => self.state.notes.view().map(Message::Notes),
        }
    }
}
