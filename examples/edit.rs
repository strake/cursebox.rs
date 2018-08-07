extern crate cursebox;
extern crate jemalloc;

use cursebox::{Attr, Cell, Event, Key};

const WIDTH : usize = 80;
const HEIGHT: usize = 25;

fn main() {
    let mut ui = cursebox::UI::new_in(jemalloc::Jemalloc).unwrap();
    let mut pt = (0, 0);
    let mut cells = [[Cell { ch: ' ' as _, fg: Attr::Default, bg: Attr::Default }; WIDTH]; HEIGHT];
    loop {
        ui.clear();
        for (y, row) in cells.iter().enumerate() {
            for (x, cell) in row.iter().cloned().enumerate() {
                if let Some(p) = ui.cells_mut().at_mut(x, y) { *p = cell }
            }
        }
        ui.set_cursor(pt.0, pt.1);
        ui.present();
        match ui.fetch_event(None) {
            Ok(Some(Event::Key(mod_, key))) if mod_.is_empty() => match key {
                Key::Left  => if pt.0 > 0          { pt.0 -= 1 },
                Key::Right => if pt.0 < WIDTH  - 1 { pt.0 += 1 },
                Key::Up    => if pt.1 > 0          { pt.1 -= 1 },
                Key::Down  => if pt.1 < HEIGHT - 1 { pt.1 += 1 },
                Key::Char('\x03') => return,
                Key::Char('\n') | Key::Char('\r') => {
                    pt.0 = 0;
                    pt.1 += 1;
                    if pt.1 >= HEIGHT { pt.1 -= HEIGHT }
                },
                Key::Char('\x08') => {
                    if pt.0 > 0 { pt.0 -= 1 }
                    cells[pt.1][pt.0].ch = ' ' as _;
                },
                Key::Char(x) => {
                    cells[pt.1][pt.0].ch = x as _;
                    if pt.0 < WIDTH - 1 { pt.0 += 1 }
                },
                _ => ()
            }
            _ => ()
        }
    }
}
