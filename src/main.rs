extern crate iui;
use iui::controls::{Entry, Label, VerticalBox};
use iui::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

mod items;

struct State {
    search_text: String,
    has_new_text: bool,
}

fn main() {
    let ui = UI::init().unwrap();

    let state = Rc::new(RefCell::new(State {
        search_text: String::new(),
        has_new_text: false,
    }));

    let mut vbox = VerticalBox::new(&ui);
    vbox.set_padded(&ui, true);

    let mut entry = Entry::new(&ui);
    vbox.append(&ui, entry.clone(), LayoutStrategy::Compact);

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
            state.has_new_text = true;
        }
    });

    let mut event_loop = ui.event_loop();
    event_loop.on_tick(&ui, {
        let ui = ui.clone();
        let mut text_label = text_label.clone();
        move || {
            let mut state = state.borrow_mut();
            if state.has_new_text {
                let items = items::get_matching(&state.search_text);
                let items = items.join("\n");

                text_label.set_text(&ui, &items);
                state.has_new_text = false;
            }
        }
    });
    event_loop.run(&ui);
}
