/// Namespace for creating rust-tui components
use std::cmp;
use tui::{
    style::{Color,Style},
    text::Span,
    widgets::{Cell, Row},
};
use super::op;

/// Given a vec of column display names, return a tui Row object
pub fn new_header_row<'a>(headers: &'a Vec<String>) -> Row<'a> {
    let header_cells = headers
        .iter()
        .map(|h| Cell::from(Span::raw(h)).style(Style::default().fg(Color::Red)));
    Row::new(header_cells)
        .style(Style::default().bg(Color::Blue))
        .height(1)
        .bottom_margin(1)
}

pub fn new_item_list_row<'a, 'b>(item: &'a op::ItemListEntry, headers: &'b Vec<String>) -> Row<'a> {
    let mut height = 1;
    let cells = headers.iter().map(|header| {
        let val = match header.as_str() {
            "id" => &item.id,
            "title" => &item.title,
            "updated_at" => &item.updated_at,
            _ => "",
        };
        height = cmp::max(height, val.chars().filter(|c| *c == '\n').count());
        Cell::from(Span::raw(val))
    });
    Row::new(cells).height(height as u16).bottom_margin(1)
}
