use std::{cmp::min, collections::HashMap, fs::File, io::{self, Read}};

use ratatui::{crossterm::{event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEventKind}, execute}, layout::Size, style::Color, text::Text};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget, DefaultTerminal, Frame};

const SIZE: Size = Size {
    width: 6,
    height: 3,
};

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

    fn init_from_file(fh: &mut File) -> io::Result<Self> {
        let mut content = String::new();
        fh.read_to_string(&mut content)?;
        let mut klots: HashMap<char, Klot> = HashMap::new();
        let mut width = 0;
        let mut height = 0;
        let mut c_idx = 1; // skip black
        for (fy, line) in content.lines().enumerate() {
            for (fx, c) in line.char_indices() {
                match c {
                    '+' | '-' | '|' | ' ' => {}
                    '_' => {
                        // todo
                    }
                    _ => {
                        let x = (fx - 1) as u16;
                        let y = (fy - 1) as u16;
                        if let Some(klot) = klots.get_mut(&c) {
                            klot.w = x + 1 - klot.x;
                            klot.h = y + 1 - klot.y;
                        } else {
                            klots.insert(c, Klot { x, y, w: 1, h: 1, color: Color::Indexed(c_idx)});
                            c_idx += 1;
                        }
                    }
                }
                width = fx;
            }
            height = fy;
        }
        Ok(Self::init(klots.into_values().collect(), width as u16 - 1, height as u16 - 1))
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
                let mx = ev.column / SIZE.width;
                let my = ev.row / SIZE.height;
                match ev.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        if let Some(idx) = self.get_klot(mx, my) {
                            self.selected = Some((idx, mx - self.klots[idx].x, my - self.klots[idx].y));
                        } else {
                            self.selected = None;
                        }
                    }
                    MouseEventKind::Drag(MouseButton::Left) => {
                        if let Some((klot_idx, off_x, off_y)) = self.selected {
                            let x = if mx > off_x {mx - off_x} else {0};
                            let y = if my > off_y {my - off_y} else {0};
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
            let x_start = klot.x * SIZE.width;
            let y_start = klot.y * SIZE.height;
            let x_end = (klot.x + klot.w) * SIZE.width;
            let y_end = (klot.y + klot.h) * SIZE.height;
            for x in x_start..x_end {
                for y in y_start..y_end {
                    let c = buf.cell_mut((x, y)).unwrap();
                    c.set_bg(klot.color);
                }
            }
        }
        // Text::raw(&self.debug).render(Rect::new(area.x, area.y + 5, area.width, area.height - 5), buf);
    }
}

fn main() -> io::Result<()> {
    let mut args = std::env::args();

    if args.len() != 2 {
        println!("Usage: {} <challenge_file>", args.next().unwrap());
        return Ok(())
    }
    args.next();

    let filename = args.next().unwrap();
    let mut file = File::open(filename)?;

    let mut app = App::init_from_file(&mut file)?;

    let mut terminal = ratatui::init();
    execute!(io::stdout(), EnableMouseCapture).unwrap();
    let res = app.run(&mut terminal);
    execute!(io::stdout(), DisableMouseCapture).unwrap();
    ratatui::restore();
    res
}
