use iced::widget::{button, column, container, horizontal_space, pick_list, row, slider, text, vertical_space};
use iced::{Alignment, Element, Font, Length, Theme};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter};
use std::path::PathBuf;

const STORAGE_FILE: &str = "settings.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeOption {
    KanagawaDragon,
    Nord,
    SolarizedLight,
    SolarizedDark,
}

impl ThemeOption {
    pub const ALL: [ThemeOption; 4] = [
        ThemeOption::KanagawaDragon,
        ThemeOption::Nord,
        ThemeOption::SolarizedLight,
        ThemeOption::SolarizedDark,
    ];

    fn label(self) -> &'static str {
        match self {
            ThemeOption::KanagawaDragon => "Kanagawa Dragon",
            ThemeOption::Nord => "Nord",
            ThemeOption::SolarizedLight => "Solarized Light",
            ThemeOption::SolarizedDark => "Solarized Dark",
        }
    }

    pub fn to_theme(self) -> Theme {
        match self {
            ThemeOption::KanagawaDragon => Theme::KanagawaDragon,
            ThemeOption::Nord => Theme::Nord,
            ThemeOption::SolarizedLight => Theme::SolarizedLight,
            ThemeOption::SolarizedDark => Theme::SolarizedDark,
        }
    }
}

impl std::fmt::Display for ThemeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontOption {
    Sans,
    Serif,
    Monospace,
}

impl FontOption {
    pub const ALL: [FontOption; 3] = [FontOption::Sans, FontOption::Serif, FontOption::Monospace];

    fn label(self) -> &'static str {
        match self {
            FontOption::Sans => "Sans",
            FontOption::Serif => "Serif",
            FontOption::Monospace => "Monospace",
        }
    }

    pub fn to_font(self) -> Font {
        match self {
            FontOption::Sans => Font::DEFAULT,
            FontOption::Serif => Font::with_name("Times New Roman"),
            FontOption::Monospace => Font::with_name("Fira Code"),
        }
    }
}

impl std::fmt::Display for FontOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Message {
    ThemeChanged(ThemeOption),
    FontChanged(FontOption),
    FontSizeChanged(u16),
    Back,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsState {
    selected_theme: ThemeOption,
    selected_font: FontOption,
    font_size: u16,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self::load()
    }
}

impl SettingsState {
    fn default_values() -> Self {
        Self {
            selected_theme: ThemeOption::KanagawaDragon,
            selected_font: FontOption::Sans,
            font_size: 16,
        }
    }

    fn load() -> Self {
        match Self::load_from_disk() {
            Ok(state) => state,
            Err(error) => {
                eprintln!("Failed to load settings from disk: {error}");
                Self::default_values()
            }
        }
    }

    pub fn theme(&self) -> Theme {
        self.selected_theme.to_theme()
    }

    pub fn font(&self) -> Font {
        self.selected_font.to_font()
    }

    pub fn font_size(&self) -> u16 {
        self.font_size
    }

    pub fn update(&mut self, message: Message) {
        let mut changed = false;

        match message {
            Message::ThemeChanged(choice) => {
                if self.selected_theme != choice {
                    self.selected_theme = choice;
                    changed = true;
                }
            }
            Message::FontChanged(choice) => {
                if self.selected_font != choice {
                    self.selected_font = choice;
                    changed = true;
                }
            }
            Message::FontSizeChanged(size) => {
                let clamped = size.clamp(10, 48);
                if self.font_size != clamped {
                    self.font_size = clamped;
                    changed = true;
                }
            }
            Message::Back => {
                
            }
        }

        if changed {
            self.persist();
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let theme_picker = pick_list(
            ThemeOption::ALL,
            Some(self.selected_theme),
            Message::ThemeChanged,
        )
        .placeholder("Select theme");

        let font_picker = pick_list(
            FontOption::ALL,
            Some(self.selected_font),
            Message::FontChanged,
        )
        .placeholder("Select font");

        let font_size_slider = slider(10.0..=48.0, self.font_size as f32, |value| {
            Message::FontSizeChanged(value.round() as u16)
        });

        let preview = text("The quick brown fox jumps over the lazy dog")
            .font(self.font())
            .size(self.font_size());

        let content = column![
            row![
                text("Appearance").size(24),
                horizontal_space().width(Length::Fill),
                button(text("Back")).on_press(Message::Back),
            ]
            .align_y(Alignment::Center),
            vertical_space().height(Length::Fixed(10.0)),
            row![text("Theme"), theme_picker]
                .spacing(12)
                .align_y(Alignment::Center),
            row![text("Font"), font_picker]
                .spacing(12)
                .align_y(Alignment::Center),
            row![
                text(format!("Font size: {}pt", self.font_size)),
                font_size_slider,
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            vertical_space().height(Length::Fixed(16.0)),
            preview,
        ]
        .spacing(16)
        .width(Length::Fill);

        container(content).padding(16).into()
    }

    fn persist(&self) {
        if let Err(error) = self.save_to_disk() {
            eprintln!("Failed to save settings: {error}");
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
        PathBuf::from(STORAGE_FILE)
    }
}
