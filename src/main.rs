use std::{borrow::Cow, cmp::min, io};

use ratatui::{crossterm::{event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEventKind}, execute}, style::Color, text::Text};
use ratatui::{buffer::Buffer, layout::Rect, style::{Modifier, Style, Stylize}, symbols::{self, border}, text::{Span, ToSpan}, widgets::{Block, Borders, Paragraph, Widget}, DefaultTerminal, Frame};

#[derive(Clone)]
struct Klot {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    color: Color
}

impl Klot {
    
}

struct App {
    exit: bool,
    width: u16,
    height: u16,
    klots: Vec<Klot>,
    selected: Option<(usize, u16, u16)>,
    debug: String
}

impl App {
    fn init(klots: Vec<Klot>, width: u16, height: u16) -> Self {
        Self {
            exit: false,
            klots,
            width,
            height,
            selected: None,
            debug: String::new()
        }
    }

    fn get_klot(&self, x: u16, y: u16) -> Option<usize> {
        self.klots.iter().position(|k| x >= k.x && x < k.x + k.w && y >= k.y && y < k.y + k.h)
    }

    fn try_move(&mut self, klot_idx: usize, mut x: u16, mut y: u16) {
        let mut klot = self.klots[klot_idx].clone();
        x = min(x, self.width - klot.w);
        y = min(y, self.height - klot.h);
        self.debug = format!("Attempting to move {} from {},{} to {},{}. ", klot_idx, klot.x, klot.y, x, y);
        while x != klot.x || y != klot.y {
            if x < klot.x {
                // left
                if ((klot.y)..(klot.y + klot.h)).all(|y| self.get_klot(klot.x - 1, y).is_none()) {
                    self.klots[klot_idx].x -= 1;
                } else {
                    x = klot.x;
                }
            } else if x > klot.x {
                // right
                if ((klot.y)..(klot.y + klot.h)).all(|y| self.get_klot(klot.x + klot.w, y).is_none()) {
                    self.klots[klot_idx].x += 1;
                } else {
                    self.debug += &format!("Collision at {},{}..{}. ", klot.x + klot.w, klot.y, klot.y + klot.h);
                    x = klot.x;
                }
            }
            if y < klot.y {
                // up
                if ((klot.x)..(klot.x + klot.w)).all(|x| self.get_klot(x, klot.y - 1).is_none()) {
                    self.klots[klot_idx].y -= 1;
                } else {
                    y = klot.y;
                }
            } else if y > klot.y {
                // down
                if ((klot.x)..(klot.x + klot.w)).all(|x| self.get_klot(x, klot.y + klot.h).is_none()) {
                    self.klots[klot_idx].y += 1;
                } else {
                    y = klot.y;
                }
            }
            klot = self.klots[klot_idx].clone();
        }
        self.debug += &format!("Ended up at {x},{y}");
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        let ev = event::read()?;
        match ev {
            Event::Key(ev) => {
                match ev.code {
                    KeyCode::Esc => {self.exit = true}
                    _ => {}
                }
            }
            Event::Mouse(ev) => {
                match ev.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        if let Some(idx) = self.get_klot(ev.column, ev.row) {
                            self.selected = Some((idx, ev.column - self.klots[idx].x, ev.row - self.klots[idx].y));
                        }
                    }
                    MouseEventKind::Drag(MouseButton::Left) => {
                        if let Some((klot_idx, off_x, off_y)) = self.selected {
                            let x = if ev.column > off_x {ev.column - off_x} else {0};
                            let y = if ev.row > off_y {ev.row - off_y} else {0};
                            self.try_move(klot_idx, x, y);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for klot in &self.klots {
            for x in klot.x..(klot.x + klot.w) {
                for y in klot.y..(klot.y + klot.h) {
                    let c = buf.cell_mut((x, y)).unwrap();
                    c.set_bg(klot.color);
                }
            }
        }
        // Text::raw(&self.debug).render(Rect::new(area.x, area.y + 5, area.width, area.height - 5), buf);
    }
}

fn main() -> io::Result<()> {
    let mut app = App::init(vec![
        Klot { x:1,y:0, w:2,h:2, color: Color::Cyan },
        Klot { x:0,y:0, w:1,h:2, color: Color::Green },
        Klot { x:3,y:0, w:1,h:2, color: Color::Blue },
        Klot { x:0,y:2, w:1,h:2, color: Color::Red },
        Klot { x:3,y:2, w:1,h:2, color: Color::Yellow },
        Klot { x:1,y:2, w:2,h:1, color: Color::Magenta },
        Klot { x:1,y:3, w:1,h:1, color: Color::White },
        Klot { x:2,y:3, w:1,h:1, color: Color::LightGreen },
        Klot { x:0,y:4, w:1,h:1, color: Color::DarkGray },
        Klot { x:3,y:4, w:1,h:1, color: Color::LightRed },
    ], 4, 5);
    let mut terminal = ratatui::init();
    execute!(io::stdout(), EnableMouseCapture).unwrap();
    let res = app.run(&mut terminal);
    execute!(io::stdout(), DisableMouseCapture).unwrap();
    ratatui::restore();
    res
}
