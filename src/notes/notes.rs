use iced::widget::button::Status as ButtonStatus;
use iced::widget::markdown::{self, Item, Settings, Style};
use iced::widget::{
    button, column, container, horizontal_space, mouse_area, row, scrollable, text, text_input,
    vertical_space,
};
use iced::{Color, Element, Length, Theme, alignment::Alignment};
use iced::{Shadow, border};
use serde::{Serialize, Deserialize};
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter};
use std::path::PathBuf;
use std::time::{Duration, Instant};

const DOUBLE_CLICK_WINDOW: Duration = Duration::from_millis(300);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoteColor {
    Default,
    Cherry,
    Emerald,
    Ocean,
    Amber,
    Violet,
}

impl NoteColor {
    const ALL: [NoteColor; 6] = [
        NoteColor::Default,
        NoteColor::Cherry,
        NoteColor::Emerald,
        NoteColor::Ocean,
        NoteColor::Amber,
        NoteColor::Violet,
    ];

    fn swatch(self) -> Option<Color> {
        match self {
            NoteColor::Default => None,
            NoteColor::Cherry => Some(Color::from_rgb8(0xf5, 0x6a, 0x6a)),
            NoteColor::Emerald => Some(Color::from_rgb8(0x5b, 0xc0, 0x7a)),
            NoteColor::Ocean => Some(Color::from_rgb8(0x4a, 0x90, 0xe2)),
            NoteColor::Amber => Some(Color::from_rgb8(0xf1, 0xc4, 0x0f)),
            NoteColor::Violet => Some(Color::from_rgb8(0xb4, 0x79, 0xe6)),
        }
    }

    fn label(self) -> &'static str {
        match self {
            NoteColor::Default => "Default",
            NoteColor::Cherry => "Cherry",
            NoteColor::Emerald => "Emerald",
            NoteColor::Ocean => "Ocean",
            NoteColor::Amber => "Amber",
            NoteColor::Violet => "Violet",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub color: NoteColor,
    #[serde(skip, default)]
    parsed: Vec<Item>, // not persisted; rebuilt from body
}

impl Note {
    pub fn new(title: String, body: String, tags: Vec<String>) -> Self {
        let parsed = markdown::parse(body.as_str()).collect();
        Self {
            title,
            body,
            tags,
            color: NoteColor::Default,
            parsed,
        }
    }

    pub fn parsed(&self) -> &[Item] {
        &self.parsed
    }

    pub fn set_color(&mut self, color: NoteColor) {
        self.color = color;
    }

    fn matches(&self, query: &str) -> bool {
        if query.is_empty() {
            true
        } else {
            self.title.to_lowercase().contains(query)
        }
    }
}

impl Default for Note {
    fn default() -> Self {
        Self::new(String::new(), String::new(), Vec::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notes {
    #[serde(skip, default)]
    selected: Option<usize>,
    entries: Vec<Note>,
    #[serde(skip, default)]
    search: String,
    #[serde(skip, default)]
    color_menu: Option<usize>,
    #[serde(skip, default)]
    last_click: Option<(usize, Instant)>,
}

#[derive(Debug, Clone)]
pub enum Message {
    NoteClicked(usize),
    CreateNew,
    LinkClicked,
    ToggleColorMenu(usize),
    ColorPicked { index: usize, color: NoteColor },
    DeleteRequested(usize),
    SearchChanged(String),
    OpenSettings,
    OpenInNewWindow(usize),
}

#[derive(Debug, Clone)]
pub enum Event {
    Edit(usize),
    Create,
    Delete(usize),
    OpenSettings,
    OpenInNewWindow(usize),
}

impl Notes {
    pub fn load() -> Self {
        match Self::load_from_disk() {
            Ok(mut notes) => {
                // Refresh parsed markdown for all notes
                for note in &mut notes.entries {
                    note.refresh_parsed();
                }
                notes
            }
            Err(error) => {
                eprintln!("Failed to load notes from disk: {error}");
                Self::default_values()
            }
        }
    }

    fn default_values() -> Self {
        Self {
            selected: None,
            entries: Vec::new(),
            search: String::new(),
            color_menu: None,
            last_click: None,
        }
    }

    fn persist(&self) {
        if let Err(error) = self.save_to_disk() {
            eprintln!("Failed to save notes: {error}");
        }
    }

    fn save_to_disk(&self) -> io::Result<()> {
        let path = Self::storage_path();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)
            .map_err(|error| io::Error::new(io::ErrorKind::Other, error))
    }

    fn load_from_disk() -> io::Result<Self> {
        let path = Self::storage_path();
        if !path.exists() {
            return Ok(Self::default_values());
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }

    fn storage_path() -> PathBuf {
        PathBuf::from("notes.json")
    }

    fn changed(&mut self) {
        self.persist();
    }

    pub fn update(&mut self, message: Message) -> Option<Event> {
        match message {
            Message::NoteClicked(index) => {
                let now = Instant::now();
                let double_click = self
                    .last_click
                    .and_then(|(last_index, time)| {
                        (last_index == index && now.duration_since(time) <= DOUBLE_CLICK_WINDOW)
                            .then_some(())
                    })
                    .is_some();

                self.last_click = Some((index, now));
                self.color_menu = None;
                self.selected = Some(index);

                if double_click {
                    self.last_click = None;
                    Some(Event::Edit(index))
                } else {
                    None
                }
            }
            Message::CreateNew => {
                self.color_menu = None;
                self.last_click = None;
                Some(Event::Create)
            }
            Message::OpenSettings => {
                Some(Event::OpenSettings)
            }
            Message::LinkClicked => None,
            Message::ToggleColorMenu(index) => {
                self.color_menu = if self.color_menu == Some(index) {
                    None
                } else {
                    Some(index)
                };
                None
            }
            Message::ColorPicked { index, color } => {
                if let Some(note) = self.entries.get_mut(index) {
                    note.set_color(color);
                }
                self.color_menu = None;
                self.changed();
                None
            }
            Message::DeleteRequested(index) => {
                if index < self.entries.len() {
                    self.entries.remove(index);
                    self.adjust_after_remove(index);
                    self.color_menu = None;
                    self.last_click = None;
                    self.changed();
                    Some(Event::Delete(index))
                } else {
                    None
                }
            }
            Message::SearchChanged(query) => {
                self.search = query;
                self.color_menu = None;
                self.last_click = None;
                None
            }
            Message::OpenInNewWindow(index) => Some(Event::OpenInNewWindow(index)),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let markdown_style = Style::from_palette(Theme::KanagawaDragon.palette());
        let query = self.search.to_lowercase();

        let mut search_row = row![
            text_input("Search titles...", &self.search)
                .on_input(Message::SearchChanged)
                .padding(10)
                .size(16)
                .width(Length::Fill)
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        if !self.search.is_empty() {
            search_row = search_row
                .push(button(text("Clear")).on_press(Message::SearchChanged(String::new())));
        }

        let mut content = column![search_row, vertical_space().height(Length::Fixed(12.0))];

        let header = row![
            text("Notebook").size(32),
            horizontal_space().width(Length::Fill),
            button(text("New page")).on_press(Message::CreateNew),
            button(text("Settings")).on_press(Message::OpenSettings),
        ]
        .align_y(Alignment::Center)
        .spacing(12);

        content = content.push(header);
        content = content.push(vertical_space().height(Length::Fixed(16.0)));

        let mut any_visible = false;

        for (index, note) in self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, note)| note.matches(&query))
        {
            any_visible = true;

            let bar_color = note.color.swatch();
            let selected = self.selected == Some(index);

            let drag_icon = container(text("=").size(18))
                .width(Length::Fixed(28.0))
                .padding([8, 0]);

            let mut note_column = column![
                row![
                    text(if note.title.trim().is_empty() {
                        "Untitled page"
                    } else {
                        note.title.as_str()
                    })
                    .size(26),
                    horizontal_space().width(Length::Fill),
                    button(text("Open in new window").size(18))
                        .on_press(Message::OpenInNewWindow(index))
                        .padding([6, 10]),
                    button(text("colors").size(18))
                        .on_press(Message::ToggleColorMenu(index))
                        .padding([6, 10]),
                    button(text("trash").size(18))
                        .on_press(Message::DeleteRequested(index))
                        .padding([6, 10]),
                ]
                .align_y(Alignment::Center)
                .spacing(6)
            ]
            .spacing(8);

            if !note.tags.is_empty() {
                let tags = note
                    .tags
                    .iter()
                    .map(|tag| format!("#{tag}"))
                    .collect::<Vec<_>>()
                    .join(" ");

                note_column = note_column.push(text(tags).size(14));
            }

            let preview = markdown::view(note.parsed(), Settings::default(), markdown_style)
                .map(|_| Message::LinkClicked);

            note_column = note_column.push(preview);

            if self.color_menu == Some(index) {
                let palette = NoteColor::ALL.iter().fold(row![], |row, color| {
                    let swatch_color = color.swatch();
                    let selected_color = note.color == *color;
                    let label = text(color.label()).size(14);

                    let swatch = container(vertical_space().height(Length::Fixed(20.0)))
                        .width(Length::Fixed(32.0))
                        .style(move |_| swatch_style(swatch_color, selected_color));

                    let button = button(column![swatch, label])
                        .padding(6)
                        .style(move |_, status| color_button_style(selected_color, status))
                        .on_press(Message::ColorPicked {
                            index,
                            color: *color,
                        });

                    row.push(button)
                });

                note_column = note_column.push(palette.spacing(8).align_y(Alignment::Center));
            }

            let card = container(note_column.spacing(10))
                .width(Length::Fill)
                .padding(16)
                .height(Length::Fixed(150.0))
                .style(move |_| note_card_style(bar_color, selected));

            let card_area: Element<'_, Message> = mouse_area(card)
                .on_press(Message::NoteClicked(index))
                .into();

            let color_bar = container(vertical_space().height(Length::Fixed(25.0)))
                .width(Length::Fixed(4.0))
                .style(move |_| color_bar_style(bar_color));

            content = content.push(
                row![drag_icon, color_bar, card_area]
                    .spacing(12)
                    .align_y(Alignment::Center),
            );
        }

        if !any_visible {
            content =
                content.push(container(text("No notes match your search.").size(16)).padding(24));
        }

        let scroll = scrollable(content.spacing(12))
            .height(Length::Fill)
            .width(Length::Fill);

        container(scroll)
            .width(Length::Fill)
            .padding([24, 32])
            .into()
    }

    pub fn select(&mut self, selection: Option<usize>) {
        self.selected = selection.and_then(|index| self.entries.get(index).map(|_| index));
    }

    pub fn upsert(&mut self, mut note: Note, editing: Option<usize>) -> usize {
        if let Some(index) = editing {
            if let Some(slot) = self.entries.get_mut(index) {
                note.color = slot.color;
                *slot = note;
                self.changed();
                index
            } else {
                let index = self.entries.len();
                self.entries.push(note);
                self.changed();
                index
            }
        } else {
            let index = self.entries.len();
            self.entries.push(note);
            self.changed();
            index
        }
    }

    pub fn get(&self, index: usize) -> Option<&Note> {
        self.entries.get(index)
    }

    fn adjust_after_remove(&mut self, index: usize) {
        if let Some(selected) = self.selected {
            if selected == index {
                self.selected = None;
            } else if selected > index {
                self.selected = Some(selected - 1);
            }
        }

        if let Some(menu) = self.color_menu {
            if menu == index {
                self.color_menu = None;
            } else if menu > index {
                self.color_menu = Some(menu - 1);
            }
        }
    }
}

impl Default for Notes {
    fn default() -> Self {
        Self::load()
    }
}

impl Note {
    fn refresh_parsed(&mut self) {
        self.parsed = markdown::parse(self.body.as_str()).collect();
    }
}

fn note_card_style(color: Option<Color>, selected: bool) -> container::Style {
    let mut style = container::Style::default();

    if let Some(color) = color {
        style = style.background(Color { a: 0.18, ..color });
    }

    style.border = border::Border {
        color: if selected {
            Color::from_rgb8(0xff, 0xff, 0xff)
        } else {
            Color::from_rgba(1.0, 1.0, 1.0, 0.08)
        },
        width: if selected { 2.0 } else { 1.0 },
        radius: border::Radius::from(8.0),
    };

    if selected {
        let mut shadow = Shadow::default();
        shadow.color = Color::from_rgba(0.0, 0.0, 0.0, 0.25);
        shadow.blur_radius = 8.0;
        style.shadow = shadow;
    }

    style
}

fn color_bar_style(color: Option<Color>) -> container::Style {
    color
        .map(container::Style::from)
        .unwrap_or_else(container::Style::default)
}

fn swatch_style(color: Option<Color>, selected: bool) -> container::Style {
    let mut style = container::Style::default();
    let base = color.unwrap_or(Color::from_rgb8(0x44, 0x44, 0x44));
    style = style.background(base);
    style.border = border::Border {
        color: if selected {
            Color::from_rgb8(0xff, 0xff, 0xff)
        } else {
            Color::TRANSPARENT
        },
        width: if selected { 2.0 } else { 0.0 },
        radius: border::Radius::from(6.0),
    };
    style
}

fn color_button_style(selected: bool, status: ButtonStatus) -> button::Style {
    let mut style = button::Style::default();
    style.text_color = Color::from_rgb8(0xee, 0xee, 0xee);
    style.border = border::Border {
        color: if selected || matches!(status, ButtonStatus::Hovered) {
            Color::from_rgb8(0xff, 0xff, 0xff)
        } else {
            Color::from_rgba(1.0, 1.0, 1.0, 0.2)
        },
        width: if selected { 2.0 } else { 1.0 },
        radius: border::Radius::from(8.0),
    };
    style
}
