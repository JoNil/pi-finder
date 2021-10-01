extern crate iui;
use iui::controls::{
    Entry, Group, HorizontalBox, HorizontalSeparator, Label, MultilineEntry, PasswordEntry,
    ProgressBar, Slider, Spacer, Spinbox, VerticalBox,
};
use iui::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

struct State {
    search_text: String,
}

fn main() {
    let ui = UI::init().unwrap();

    let state = Rc::new(RefCell::new(State {
        search_text: String::new(),
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
            state.borrow_mut().search_text = val;
        }
    });

    let mut event_loop = ui.event_loop();
    event_loop.on_tick(&ui, {
        let ui = ui.clone();
        let mut text_label = text_label.clone();
        move || {
            let state = state.borrow();
            text_label.set_text(&ui, &format!("{}\n{}", state.search_text, state.search_text));
        }
    });
    event_loop.run(&ui);
}
