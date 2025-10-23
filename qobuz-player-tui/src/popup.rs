use qobuz_player_models::{AlbumSimple, Playlist};
use ratatui::{crossterm::event::KeyCode, prelude::*, widgets::*};

use crate::{
    app::PlayOutcome,
    ui::{block, center},
};

#[derive(PartialEq)]
pub(crate) struct ArtistPopupState {
    pub artist_name: String,
    pub albums: Vec<AlbumSimple>,
    pub state: ListState,
}

#[derive(PartialEq)]
pub(crate) struct PlaylistPopupState {
    pub playlist_name: String,
    pub playlist_id: u32,
    pub shuffle: bool,
}

#[derive(PartialEq)]
pub(crate) struct QueuePopupState {
    pub track_id: u32,
    pub playlists: Vec<Playlist>,
    pub state: ListState,
}

#[derive(PartialEq)]
pub(crate) enum Popup {
    Artist(ArtistPopupState),
    Playlist(PlaylistPopupState),
    QueueAdd(QueuePopupState),
    QueueDelete(QueuePopupState),
}

impl Popup {
    pub(crate) fn render(&mut self, frame: &mut Frame) {
        match self {
            Popup::Artist(artist) => {
                let area = center(
                    frame.area(),
                    Constraint::Percentage(50),
                    Constraint::Length(artist.albums.len() as u16 + 2),
                );

                let list: Vec<ListItem> = artist
                    .albums
                    .iter()
                    .map(|album| ListItem::from(Line::from(album.title.clone())))
                    .collect();

                let list = List::new(list)
                    .block(block(&artist.artist_name, false))
                    .highlight_style(Style::default().bg(Color::Blue))
                    .highlight_symbol(">")
                    .highlight_spacing(HighlightSpacing::Always);

                frame.render_widget(Clear, area);
                frame.render_stateful_widget(list, area, &mut artist.state);
            }
            Popup::Playlist(playlist) => {
                let area = center(frame.area(), Constraint::Length(18), Constraint::Length(3));
                let tabs = Tabs::new(["Play", "Shuffle"])
                    .block(block(&playlist.playlist_name, false))
                    .not_underlined()
                    .highlight_style(Style::default().bg(Color::Blue))
                    .select(if playlist.shuffle { 1 } else { 0 })
                    .divider(symbols::line::VERTICAL);

                frame.render_widget(Clear, area);
                frame.render_widget(tabs, area);
            }
            Popup::QueueAdd(queue) => {
                let area = center(
                    frame.area(),
                    Constraint::Percentage(50),
                    Constraint::Length(queue.playlists.len() as u16 + 2),
                );

                let list: Vec<ListItem> = queue
                    .playlists
                    .iter()
                    .map(|playlist| ListItem::from(Line::from(playlist.title.clone())))
                    .collect();

                let list = List::new(list)
                    .block(block("Add to Playlist", false))
                    .highlight_style(Style::default().bg(Color::Blue))
                    .highlight_symbol(">")
                    .highlight_spacing(HighlightSpacing::Always);

                frame.render_widget(Clear, area);
                frame.render_stateful_widget(list, area, &mut queue.state);
            }
            Popup::QueueDelete(queue) => {
                let area = center(
                    frame.area(),
                    Constraint::Percentage(50),
                    Constraint::Length(queue.playlists.len() as u16 + 2),
                );

                let list: Vec<ListItem> = queue
                    .playlists
                    .iter()
                    .map(|playlist| ListItem::from(Line::from(playlist.title.clone())))
                    .collect();

                let list = List::new(list)
                    .block(block("Delete from Playlist", false))
                    .highlight_style(Style::default().bg(Color::Blue))
                    .highlight_symbol(">")
                    .highlight_spacing(HighlightSpacing::Always);

                frame.render_widget(Clear, area);
                frame.render_stateful_widget(list, area, &mut queue.state);
            }
        };
    }

    pub(crate) async fn handle_event(&mut self, key: KeyCode) -> Option<PlayOutcome> {
        match self {
            Popup::Artist(artist_popup_state) => match key {
                KeyCode::Up | KeyCode::Char('k') => {
                    artist_popup_state.state.select_previous();
                    None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    artist_popup_state.state.select_next();
                    None
                }
                KeyCode::Enter => {
                    let index = artist_popup_state.state.selected();

                    let id = index
                        .and_then(|index| artist_popup_state.albums.get(index))
                        .map(|album| album.id.clone());

                    if let Some(id) = id {
                        return Some(PlayOutcome::Album(id));
                    }

                    None
                }
                _ => None,
            },
            Popup::Playlist(playlist_popup_state) => match key {
                KeyCode::Left | KeyCode::Char('h') => {
                    playlist_popup_state.shuffle = !playlist_popup_state.shuffle;
                    None
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    playlist_popup_state.shuffle = !playlist_popup_state.shuffle;
                    None
                }
                KeyCode::Enter => {
                    let id = playlist_popup_state.playlist_id;
                    Some(PlayOutcome::Playlist((id, playlist_popup_state.shuffle)))
                }
                _ => None,
            },
            Popup::QueueAdd(queue_popup_state) => match key {
                KeyCode::Up | KeyCode::Char('k') => {
                    queue_popup_state.state.select_previous();
                    None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    queue_popup_state.state.select_next();
                    None
                }
                KeyCode::Enter => {
                    if let Some(playlist_index) = queue_popup_state.state.selected()
                        && let Some(playlist) = queue_popup_state.playlists.get(playlist_index)
                    {
                        let playlist_id = playlist.id.to_string();
                        let track_id = queue_popup_state.track_id.to_string();

                        return Some(PlayOutcome::AddTrackToPlaylist {
                            track_id,
                            playlist_id,
                        });
                    }
                    None
                }
                _ => None,
            },
            Popup::QueueDelete(queue_popup_state) => match key {
                KeyCode::Up | KeyCode::Char('k') => {
                    queue_popup_state.state.select_previous();
                    None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    queue_popup_state.state.select_next();
                    None
                }
                KeyCode::Enter => {
                    if let Some(playlist_index) = queue_popup_state.state.selected()
                        && let Some(playlist) = queue_popup_state.playlists.get(playlist_index)
                    {
                        let playlist_id = playlist.id.to_string();
                        if let Some(playlist_track_id) = playlist
                            .playlist_track_id_map
                            .get(&queue_popup_state.track_id)
                        {
                            return Some(PlayOutcome::DeleteTrackFromPlaylist {
                                track_id: playlist_track_id.to_string(),
                                playlist_id,
                            });
                        }
                    }
                    None
                }
                _ => None,
            },
        }
    }
}
