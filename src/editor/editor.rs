use iced::widget::markdown::{self, Item, Settings, Style};
use iced::widget::text_editor::{self as editor_widget, Content};
use iced::{
    Element, Length, Theme,
    alignment::Alignment,
    widget::{
        button, column, container, row, scrollable, text, text_editor, text_input, vertical_space,
    },
};

use crate::notes::notes::Note;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    PreviewOnly,
    Split,
}

#[derive(Debug)]
pub struct Editor {
    title: String,
    tags_input: String,
    body: Content,
    preview: Vec<Item>,
    editing: Option<usize>,
    mode: ViewMode,
}

#[derive(Debug, Clone)]
pub enum Message {
    TitleChanged(String),
    TagsChanged(String),
    BodyEdited(editor_widget::Action),
    SavePressed,
    BackPressed,
    PreviewLinkClicked,
    ToggleViewMode,
}

#[derive(Debug, Clone)]
pub enum Event {
    Save {
        title: String,
        body: String,
        tags: Vec<String>,
        editing: Option<usize>,
    },
    Back,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            title: String::new(),
            tags_input: String::new(),
            body: Content::new(),
            preview: Vec::new(),
            editing: None,
            mode: ViewMode::PreviewOnly,
        }
    }
}

impl Editor {
    pub fn update(&mut self, message: Message) -> Option<Event> {
        match message {
            Message::TitleChanged(value) => {
                self.title = value;
                None
            }
            Message::TagsChanged(value) => {
                self.tags_input = value;
                None
            }
            Message::BodyEdited(action) => {
                self.body.perform(action);
                self.refresh_preview();
                None
            }
            Message::SavePressed => {
                let title = self.title.trim().to_string();
                let raw_body = self.body.text();
                let body = raw_body.trim_end_matches('\n').to_string();

                if title.is_empty() && body.is_empty() {
                    return None;
                }

                let tags = self
                    .tags_input
                    .split(',')
                    .filter_map(|tag| {
                        let trimmed = tag.trim();
                        (!trimmed.is_empty()).then(|| trimmed.to_string())
                    })
                    .collect::<Vec<_>>();

                Some(Event::Save {
                    title,
                    body,
                    tags,
                    editing: self.editing,
                })
            }
            Message::BackPressed => {
                self.load_new();
                Some(Event::Back)
            }
            Message::PreviewLinkClicked => None,
            Message::ToggleViewMode => {
                self.mode = match self.mode {
                    ViewMode::PreviewOnly => ViewMode::Split,
                    ViewMode::Split => ViewMode::PreviewOnly,
                };

                if matches!(self.mode, ViewMode::PreviewOnly) {
                    self.refresh_preview();
                }

                None
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let header = if self.editing.is_some() {
            "Edit page"
        } else {
            "New page"
        };

        let toggle_label = match self.mode {
            ViewMode::PreviewOnly => "Edit with preview",
            ViewMode::Split => "Preview only",
        };

        let save_label = if self.editing.is_some() {
            "Update page"
        } else {
            "Save page"
        };

        let layout = match self.mode {
            ViewMode::Split => self.split_layout(header, toggle_label, save_label),
            ViewMode::PreviewOnly => self.preview_layout(header, toggle_label, save_label),
        };

        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([24, 32])
            .into()
    }

    pub fn load_new(&mut self) {
        self.editing = None;
        self.title.clear();
        self.tags_input.clear();
        self.body = Content::new();
        self.preview.clear();
        self.mode = ViewMode::PreviewOnly;
    }

    pub fn load_existing(&mut self, index: usize, note: &Note) {
        self.editing = Some(index);
        self.title = note.title.to_owned();
        self.tags_input = note.tags.join(", ");
        self.body = Content::with_text(note.body.as_str());
        self.preview = note.parsed().to_vec();
        self.mode = ViewMode::PreviewOnly;
    }

    pub fn editing(&self) -> Option<usize> {
        self.editing
    }

    pub fn adjust_after_delete(&mut self, index: usize) {
        if let Some(current) = self.editing {
            if current == index {
                self.load_new();
            } else if current > index {
                self.editing = Some(current - 1);
            }
        }
    }

    fn split_layout<'a>(
        &'a self,
        header: &'a str,
        toggle_label: &'a str,
        save_label: &'a str,
    ) -> Element<'a, Message> {
        let title_input = text_input("Page title", &self.title)
            .on_input(Message::TitleChanged)
            .padding(12)
            .size(20)
            .width(Length::Fill);

        let tags_input = text_input("Tags (comma separated)", &self.tags_input)
            .on_input(Message::TagsChanged)
            .padding(12)
            .size(16)
            .width(Length::Fill);

        let body_editor = text_editor(&self.body)
            .placeholder("Fill the page with your thoughts...")
            .height(Length::Fill)
            .wrapping(text::Wrapping::WordOrGlyph)
            .on_action(Message::BodyEdited);

        let editor_panel = column![
            title_input,
            tags_input,
            container(body_editor)
                .width(Length::Fill)
                .height(Length::FillPortion(1))
                .padding(12),
        ]
        .spacing(12)
        .width(Length::Fill)
        .height(Length::Fill);

        let preview_panel = column![
            text("Preview").size(24),
            scrollable(
                container(self.preview_element())
                    .padding(12)
                    .width(Length::Fill)
                    .height(Length::Shrink),
            )
            .width(Length::Fill)
            .height(Length::Fill),
        ]
        .spacing(12)
        .width(Length::Fill)
        .height(Length::Fill);

        let actions = row![
            button("Back to notebook").on_press(Message::BackPressed),
            button(toggle_label).on_press(Message::ToggleViewMode),
            button(save_label).on_press(Message::SavePressed),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        let split = row![
            container(editor_panel)
                .width(Length::FillPortion(1))
                .height(Length::Fill)
                .padding(16),
            container(preview_panel)
                .width(Length::FillPortion(1))
                .height(Length::Fill)
                .padding(16),
        ]
        .spacing(24)
        .height(Length::Fill);

        column![
            text(header).size(28),
            vertical_space().height(Length::Fixed(12.0)),
            text("Give your page a title, tags, and as much text as you need.").size(16),
            vertical_space().height(Length::Fixed(16.0)),
            split,
            vertical_space().height(Length::Fixed(16.0)),
            actions,
        ]
        .spacing(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn preview_layout<'a>(
        &'a self,
        header: &'a str,
        toggle_label: &'a str,
        save_label: &'a str,
    ) -> Element<'a, Message> {
        let title_display = if self.title.trim().is_empty() {
            "Untitled page"
        } else {
            self.title.as_str()
        };

        let tags_display = self
            .tags_input
            .split(',')
            .filter_map(|tag| {
                let trimmed = tag.trim();
                (!trimmed.is_empty()).then(|| format!("#{trimmed}"))
            })
            .collect::<Vec<_>>()
            .join(" ");

        let mut metadata = column![text(title_display).size(28)]
            .spacing(8)
            .width(Length::Fill);

        if !tags_display.is_empty() {
            metadata = metadata.push(text(tags_display).size(16));
        }

        let actions = row![
            button("Back to notebook").on_press(Message::BackPressed),
            button(toggle_label).on_press(Message::ToggleViewMode),
            button(save_label).on_press(Message::SavePressed),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        let preview_panel = scrollable(
            container(self.preview_element())
                .padding(12)
                .width(Length::Fill)
                .height(Length::Shrink),
        )
        .width(Length::Fill)
        .height(Length::Fill);

        column![
            text(header).size(28),
            vertical_space().height(Length::Fixed(12.0)),
            metadata,
            vertical_space().height(Length::Fixed(16.0)),
            preview_panel,
            vertical_space().height(Length::Fixed(16.0)),
            actions,
        ]
        .spacing(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn refresh_preview(&mut self) {
        let text = self.body.text();
        self.preview = markdown::parse(text.as_str()).collect();
    }

    fn preview_element(&self) -> Element<'_, Message> {
        markdown::view(
            &self.preview,
            Settings::default(),
            Style::from_palette(Theme::KanagawaDragon.palette()),
        )
        .map(|_| Message::PreviewLinkClicked)
    }
}



