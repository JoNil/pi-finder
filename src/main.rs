use gtk::{
    self, gio,
    prelude::{
        ApplicationExt, ApplicationExtManual, BoxExt, ContainerExt, EntryExt, GtkWindowExt,
        LabelExt, WidgetExt,
    },
};
use std::cell::RefCell;

mod items;

struct State {
    selected_change: bool,
    selected: i32,
    last_res: Vec<String>,
    label: gtk::Label,
}

/*fn key_event(&mut self, _area: &Area, area_key_event: &AreaKeyEvent) -> bool {
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
}*/

thread_local!(
    static STATE: RefCell<Option<State>> = RefCell::new(None)
);

fn build_ui(application: &gtk::Application) {
    let entry = gtk::Entry::new();
    let label = gtk::Label::new(Some("HELLO"));

    STATE.with(|global| {
        *global.borrow_mut() = Some(State {
            selected: 0,
            selected_change: false,
            last_res: Vec::new(),
            label: label.clone(),
        });
    });

    let window = gtk::ApplicationWindow::new(application);
    window.set_title("pi-finder");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(260, 40);

    entry.connect_text_notify(|e| {
        let text = e.text();

        STATE.with(|global| {
            let mut state = global.borrow_mut();
            let state = state.as_mut().unwrap();

            state.selected = 0;

            if text.len() > 0 {
                state.last_res = items::get_matching(&text);
            } else {
                state.last_res = Vec::new();
            }

            let mut items_string = String::new();
            for (i, item) in state.last_res.iter().enumerate() {
                if i == state.selected as usize {
                    items_string.push_str(&format!("*{}\n", item));
                } else {
                    items_string.push_str(&format!("{}\n", item));
                }
            }

            state.label.set_text(&items_string);
        });
    });

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.set_spacing(6);
    vbox.pack_start(&entry, true, true, 6);
    vbox.pack_start(&label, true, true, 6);
    window.add(&vbox);
    window.show_all();
}

fn main() {
    let application = gtk::Application::new(
        Some("jonathan.pi-finder"),
        gio::ApplicationFlags::NON_UNIQUE,
    );

    application.connect_startup(build_ui);
    application.connect_activate(|_| {});

    application.run();
}
