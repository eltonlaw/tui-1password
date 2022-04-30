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

#[allow(dead_code)]
fn scramble_string(s_old: &String) -> String {
    let mut s_new = String::with_capacity(s_old.len());
    for c_old in s_old.as_str().chars() {
        s_new.push({
            let charset = match c_old {
                'a' | 'e' | 'i' | 'o' | 'u' => "aeiou",
                'A' | 'E' | 'I' | 'O' | 'U' => "AEIOU",
                'a'..='z' => "abcdefghijklmnopqrstuvwxyz",
                'A'..='Z' => "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
                '0'..='9' => "0123456789",
                _ => "?",
            };
            let chars: Vec<char> = charset.chars().collect();
            unsafe {
                *chars.get_unchecked(fastrand::usize(0..chars.len()))
            }
        });
    };
    s_new
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
