use crate::clipboard::write_entry;
use crate::entry::{ContentType, Entry};
use crate::storage::Storage;
use iced::keyboard::{self, Key, Modifiers};
use iced::widget::{column, container, row, scrollable, text, text_input, Column};
use iced::{event, Color, Element, Event, Length, Task as Command};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::to_layer_message;

const MAX_ENTRIES: usize = 50;
const PREVIEW_LEN: usize = 60;

pub fn run_popup() -> Result<(), iced_layershell::Error> {
    iced_layershell::application(PopupState::new, namespace, update, view)
        .subscription(subscription)
        .settings(Settings {
            layer_settings: LayerShellSettings {
                size: Some((500, 400)),
                anchor: Anchor::empty(),
                layer: Layer::Overlay,
                keyboard_interactivity: KeyboardInteractivity::Exclusive,
                ..Default::default()
            },
            ..Default::default()
        })
        .run()
}

#[derive(Default)]
struct PopupState {
    search: String,
    entries: Vec<Entry>,
    filtered: Vec<usize>,
    selected: usize,
}

impl PopupState {
    fn new() -> Self {
        let entries = Storage::open()
            .and_then(|s| s.list(MAX_ENTRIES))
            .unwrap_or_default();

        let filtered: Vec<usize> = (0..entries.len()).collect();

        Self {
            search: String::new(),
            entries,
            filtered,
            selected: 0,
        }
    }

    fn filter_entries(&mut self) {
        if self.search.is_empty() {
            self.filtered = (0..self.entries.len()).collect();
        } else {
            let query = self.search.to_lowercase();
            self.filtered = self
                .entries
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    e.content
                        .as_ref()
                        .map(|c| c.to_lowercase().contains(&query))
                        .unwrap_or(false)
                })
                .map(|(i, _)| i)
                .collect();
        }
        self.selected = 0;
    }

    fn selected_entry(&self) -> Option<&Entry> {
        self.filtered
            .get(self.selected)
            .and_then(|&i| self.entries.get(i))
    }

    fn confirm_selection(&self) {
        if let Some(entry) = self.selected_entry() {
            let _ = write_entry(entry);
        }
        std::process::exit(0);
    }

    fn cancel(&self) {
        std::process::exit(0);
    }
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    SearchChanged(String),
    SelectNext,
    SelectPrev,
    Confirm,
    Cancel,
    IcedEvent(Event),
}

fn namespace() -> String {
    "syo-popup".to_string()
}

fn subscription(_: &PopupState) -> iced::Subscription<Message> {
    event::listen().map(Message::IcedEvent)
}

fn update(state: &mut PopupState, message: Message) -> Command<Message> {
    match message {
        Message::SearchChanged(query) => {
            state.search = query;
            state.filter_entries();
            Command::none()
        }
        Message::SelectNext => {
            if !state.filtered.is_empty() && state.selected < state.filtered.len() - 1 {
                state.selected += 1;
            }
            Command::none()
        }
        Message::SelectPrev => {
            if state.selected > 0 {
                state.selected -= 1;
            }
            Command::none()
        }
        Message::Confirm => {
            state.confirm_selection();
            Command::none()
        }
        Message::Cancel => {
            state.cancel();
            Command::none()
        }
        Message::IcedEvent(Event::Keyboard(keyboard::Event::KeyPressed {
            key, modifiers, ..
        })) => handle_key(state, key, modifiers),
        _ => Command::none(),
    }
}

fn handle_key(state: &mut PopupState, key: Key, _modifiers: Modifiers) -> Command<Message> {
    match key {
        Key::Named(keyboard::key::Named::Escape) => {
            state.cancel();
            Command::none()
        }
        Key::Named(keyboard::key::Named::ArrowDown) => {
            if !state.filtered.is_empty() && state.selected < state.filtered.len() - 1 {
                state.selected += 1;
            }
            Command::none()
        }
        Key::Named(keyboard::key::Named::ArrowUp) => {
            if state.selected > 0 {
                state.selected -= 1;
            }
            Command::none()
        }
        Key::Named(keyboard::key::Named::Enter) => {
            state.confirm_selection();
            Command::none()
        }
        _ => Command::none(),
    }
}

fn view(state: &PopupState) -> Element<'_, Message> {
    let search_input = text_input("Search...", &state.search)
        .on_input(Message::SearchChanged)
        .padding(10)
        .size(16);

    let entries_list: Column<Message> =
        state
            .filtered
            .iter()
            .enumerate()
            .fold(Column::new().spacing(2), |col, (i, &entry_idx)| {
                let entry = &state.entries[entry_idx];
                let is_selected = i == state.selected;
                col.push(entry_row(entry, is_selected))
            });

    let content = column![
        search_input,
        scrollable(entries_list)
            .height(Length::Fill)
            .width(Length::Fill),
    ]
    .spacing(10)
    .padding(15);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.1, 0.12))),
            border: iced::Border {
                color: Color::from_rgb(0.3, 0.3, 0.35),
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn entry_row(entry: &Entry, selected: bool) -> Element<'static, Message> {
    let type_badge = match entry.content_type {
        ContentType::Text => text("TXT").size(10),
        ContentType::Link => text("URL").size(10),
        ContentType::Image => text("IMG").size(10),
    };

    let preview = text(entry.display_preview(PREVIEW_LEN)).size(14);

    let bg_color = if selected {
        Color::from_rgb(0.2, 0.25, 0.35)
    } else {
        Color::TRANSPARENT
    };

    let content = row![
        container(type_badge)
            .padding(4)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.2, 0.2, 0.25))),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        preview,
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    container(content)
        .padding(8)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg_color)),
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}
