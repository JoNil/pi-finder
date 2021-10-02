extern crate iui;
use iui::controls::{Area, AreaHandler, AreaKeyEvent, Entry, Label, VerticalBox};
use iui::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

mod items;

struct State {
    search_text: String,
    search_text_changed: bool,
    selected_change: bool,
    selected: i32,
    last_res: Vec<String>,
}

struct Handler {
    state: Rc<RefCell<State>>,
}
impl AreaHandler for Handler {
    fn key_event(&mut self, _area: &Area, area_key_event: &AreaKeyEvent) -> bool {
        let mut res = false;
        let mut state = self.state.borrow_mut();

        if state.last_res.len() == 0 {
            return false;
        }

        if area_key_event.up && area_key_event.ext_key == 9 && state.selected == 0 {
            state.selected = 1;
            state.selected_change = true;
        } else if !area_key_event.up {
            if area_key_event.ext_key == 9 && state.selected < (state.last_res.len() as i32) - 1 {
                state.selected += 1;
                state.selected_change = true;
            } else if area_key_event.ext_key == 8 && state.selected > 0 {
                state.selected -= 1;
                state.selected_change = true;
                if state.selected > 0 {
                    res = true;
                }
            }
        }

        res
    }
}

fn main() {
    let ui = UI::init().unwrap();

    let state = Rc::new(RefCell::new(State {
        search_text: String::new(),
        search_text_changed: false,
        selected: 0,
        selected_change: false,
        last_res: Vec::new(),
    }));

    let mut vbox = VerticalBox::new(&ui);
    vbox.set_padded(&ui, true);

    let mut entry = Entry::new(&ui);
    vbox.append(&ui, entry.clone(), LayoutStrategy::Compact);

    let area = Area::new(
        &ui,
        Box::new(Handler {
            state: state.clone(),
        }),
    );
    vbox.append(&ui, area.clone(), LayoutStrategy::Compact);

    let text_label = Label::new(&ui, "");
    vbox.append(&ui, text_label.clone(), LayoutStrategy::Compact);

    let mut window = Window::new(&ui, "Pi Finder", 300, 150, WindowType::NoMenubar);
    window.set_child(&ui, vbox);
    window.show(&ui);

    entry.on_changed(&ui, {
        let state = state.clone();
        move |val| {
            let mut state = state.borrow_mut();
            state.search_text = val;
            state.search_text_changed = true;
        }
    });

    let mut event_loop = ui.event_loop();
    event_loop.on_tick(&ui, {
        let ui = ui.clone();
        let mut text_label = text_label.clone();
        move || {
            let mut state = state.borrow_mut();
            if state.search_text_changed {
                state.search_text_changed = false;
                state.selected = 0;

                if state.search_text.len() > 0 {
                    state.last_res = items::get_matching(&state.search_text);
                } else {
                    state.last_res = Vec::new();
                }

                state.selected_change = true;
            }

            if state.selected_change {
                state.selected_change = false;

                let mut items_string = String::new();
                for (i, item) in state.last_res.iter().enumerate() {
                    if i == state.selected as usize {
                        items_string.push_str(&format!("*{}\n", item));
                    } else {
                        items_string.push_str(&format!("{}\n", item));
                    }
                }

                text_label.set_text(&ui, &items_string);
            }
        }
    });
    event_loop.run(&ui);
}
