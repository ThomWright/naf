use std::io;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use tui::{Frame, Terminal};

use crate::app;

pub fn draw<B: Backend>(
  terminal: &mut Terminal<B>,
  state: &app::State,
) -> io::Result<()> {
  terminal.draw(|f| {
    let frame_chunks = Layout::default()
      .direction(Direction::Vertical)
      .margin(0)
      .constraints(
        [
          Constraint::Min(5),
          // Should be 1, but: https://github.com/fdehau/tui-rs/issues/366
          Constraint::Length(2),
        ]
        .as_ref(),
      )
      .split(f.size());

    let files_block = Block::default().title("Files").borders(Borders::ALL);
    let files_block_inner_area = files_block.inner(frame_chunks[0]);
    f.render_widget(files_block, frame_chunks[0]);
    {
      let list_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints(
          [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
        )
        .split(files_block_inner_area);

      draw_file_list(f, list_chunks[0], &state, 0);
      draw_file_list(f, list_chunks[1], &state, 1);
    }

    draw_status(f, frame_chunks[1], &state);
  })
}

fn draw_file_list<B: Backend>(
  f: &mut Frame<B>,
  area: Rect,
  state: &app::State,
  thing: usize,
) {
  let list_block = Block::default().borders(Borders::NONE);

  let mut list_state = ListState::default();
  list_state.select(state.selected_file_in_list(thing));
  let files = state.files_in_list(thing);
  let list = (match files {
    None => List::new(vec![]),
    Some(files) => List::new(
      files
        .iter()
        .map(|f| {
          ListItem::new(vec![Spans::from(vec![if f.is_dir() {
            Span::styled(
              format!("{:<9}", &f.name()),
              Style::default()
                .fg(Color::LightBlue)
                .add_modifier(Modifier::BOLD),
            )
          } else {
            Span::raw(f.name())
          }])])
        })
        .collect::<Vec<ListItem>>(),
    ),
  })
  .block(list_block)
  .highlight_style(Style::default().bg(Color::Rgb(32, 32, 32)));

  f.render_stateful_widget(list, area, &mut list_state);
}

fn draw_status<B: Backend>(f: &mut Frame<B>, area: Rect, state: &app::State) {
  let status_block = Block::default().borders(Borders::NONE);
  let para = Paragraph::new(state.base_path())
    .block(status_block)
    .alignment(Alignment::Left);
  f.render_widget(para, area);
}
