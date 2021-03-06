use gtk::{
    self,
    gdk::EventMask,
    gio,
    glib::timeout_add,
    prelude::{
        ApplicationExt, ApplicationExtManual, BoxExt, ContainerExt, Continue, EntryExt,
        GtkWindowExt, LabelExt, WidgetExt, WidgetExtManual,
    },
    Inhibit,
};
use items::Item;
use once_cell::sync::Lazy;
use std::{
    cell::RefCell,
    cmp::{max, min},
    thread,
    time::Duration,
};

mod items;

static ITEMS: Lazy<Vec<Item>> = Lazy::new(|| items::get());

struct State {
    application: gtk::Application,
    window: gtk::ApplicationWindow,
    label: gtk::Label,
    activated: bool,
    last_res: Vec<&'static Item>,
    selected: i32,
}

impl State {
    fn update_label_text(&mut self) {
        self.selected = self.selected.clamp(0, self.last_res.len() as i32);

        let mut items_string = String::new();
        for (i, item) in self.last_res.iter().enumerate() {
            if i == self.selected as usize {
                items_string.push_str(&format!("*{}\n", item));
            } else {
                items_string.push_str(&format!("{}\n", item));
            }
        }

        self.label.set_text(&items_string);
    }
}

thread_local!(
    static STATE: RefCell<Option<State>> = RefCell::new(None)
);

fn build_ui(application: &gtk::Application) {
    let entry = gtk::Entry::new();
    let label = gtk::Label::new(Some(""));
    label.set_xalign(0.0);

    let window = gtk::ApplicationWindow::new(application);
    window.set_title("pi-finder");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::None);
    window.set_default_size(260, 40);
    window.set_decorated(false);
    window.set_skip_taskbar_hint(true);
    window.set_keep_above(true);
    window.move_(0, 36);

    WidgetExtManual::add_events(
        &window,
        EventMask::KEY_PRESS_MASK | EventMask::KEY_RELEASE_MASK | EventMask::FOCUS_CHANGE_MASK,
    );

    STATE.with(|global| {
        *global.borrow_mut() = Some(State {
            application: application.clone(),
            window: window.clone(),
            label: label.clone(),
            activated: false,
            last_res: Vec::new(),
            selected: 0,
        });
    });

    window.connect_focus_out_event(|_, e| {
        STATE.with(|global| {
            let mut state = global.borrow_mut();
            let state = state.as_mut().unwrap();

            state.window.hide();

            if !e.is_in() {
                timeout_add(Duration::from_millis(250), || {
                    STATE.with(|global| {
                        let mut state = global.borrow_mut();
                        let state = state.as_mut().unwrap();
                        state.application.quit();
                    });
                    Continue(false)
                });
            }
        });
        Inhibit(false)
    });

    window.connect_key_press_event(|_, event| -> Inhibit {
        STATE.with(|global| {
            let mut state = global.borrow_mut();
            let state = state.as_mut().unwrap();
            match event.keycode() {
                Some(9) => state.application.quit(),
                Some(36) => {
                    if let Some(item) = state.last_res.get(state.selected as usize) {
                        item.execute();
                        state.application.quit();
                    }
                }
                Some(111) => {
                    state.selected = max(state.selected - 1, 0);
                    state.update_label_text();
                }
                Some(116) => {
                    state.selected = min(state.selected + 1, state.last_res.len() as i32);
                    state.update_label_text();
                }
                _ => (),
            }
        });

        Inhibit(false)
    });

    entry.connect_text_notify(|e| {
        let text = e.text();

        STATE.with(|global| {
            let mut state = global.borrow_mut();
            let state = state.as_mut().unwrap();

            state.window.resize(260, 1);

            if text.len() > 0 {
                state.last_res = items::filter(&ITEMS, &text);
            } else {
                state.last_res = Vec::new();
            }

            state.update_label_text();
        });
    });

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.set_spacing(6);
    vbox.pack_start(&entry, false, true, 6);
    vbox.pack_start(&label, false, true, 6);

    window.add(&vbox);
    window.show_all();
}

fn main() {
    let application = gtk::Application::new(
        Some("name.jonathan.pi-finder"),
        gio::ApplicationFlags::empty(),
    );

    application.connect_startup(build_ui);
    application.connect_activate(|_| {
        STATE.with(|global| {
            thread::spawn(|| {
                Lazy::force(&ITEMS);
            });

            let mut state = global.borrow_mut();
            let state = state.as_mut().unwrap();

            if state.activated {
                state.application.quit();
                return;
            }

            let window = state.window.window().unwrap();
            window.raise();
            window.focus(0);

            state.activated = true;
        });
    });
    application.run();
}
