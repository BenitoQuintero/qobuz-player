use qobuz_player_models::{Playlist, Track, TrackStatus};
use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    prelude::*,
    style::Styled,
    widgets::*,
};

use crate::{
    app::{Output, PlayOutcome, UnfilteredListState},
    popup::{Popup, QueuePopupState},
    ui::basic_list_table,
};

pub(crate) struct QueueState {
    pub queue: UnfilteredListState<Track>,
    pub playlists: UnfilteredListState<Playlist>,
}

impl QueueState {
    pub(crate) fn render(&mut self, frame: &mut Frame, area: Rect) {
        let table = basic_list_table(
            self.queue
                .items
                .iter()
                .enumerate()
                .map(|(index, track)| {
                    let style = match track.status {
                        TrackStatus::Played => Style::default().add_modifier(Modifier::CROSSED_OUT),
                        TrackStatus::Playing => Style::default().add_modifier(Modifier::BOLD),
                        TrackStatus::Unplayed => Style::default(),
                        TrackStatus::Unplayable => {
                            Style::default().add_modifier(Modifier::CROSSED_OUT)
                        }
                    };
                    Row::new(Line::from(vec![
                        format!("{} {}", index + 1, track.title.clone()).set_style(style),
                    ]))
                })
                .collect(),
            " Queue ",
        );

        frame.render_stateful_widget(table, area, &mut self.queue.state);
    }

    pub(crate) async fn handle_events(&mut self, event: Event) -> Output {
        match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.queue.state.select_next();
                        Output::Consumed
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.queue.state.select_previous();
                        Output::Consumed
                    }
                    KeyCode::Enter => {
                        let index = self.queue.state.selected();

                        if let Some(index) = index {
                            return Output::PlayOutcome(PlayOutcome::SkipToPosition(index as u32));
                        }
                        Output::Consumed
                    }
                    KeyCode::Char('a') => {
                        let track_index = self.queue.state.selected();

                        let id = track_index
                            .and_then(|index| self.queue.items.get(index))
                            .map(|track| track.id);

                        // HACK: Probably shouldnt clone this vec, runs everytime
                        let user_playlists = self.playlists.items.clone();

                        if let Some(id) = id {
                            return Output::Popup(Popup::Queue(QueuePopupState {
                                track_id: id,
                                playlists: user_playlists,
                                state: Default::default(),
                            }));
                        }
                        Output::Consumed
                    }
                    _ => Output::NotConsumed,
                }
            }
            _ => Output::NotConsumed,
        }
    }
}
