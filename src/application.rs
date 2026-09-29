use crate::{
    browser::{
        recovery::wrap_with_recovery,
        state::WindowState,
        window::{chatgpt_web_view, stop_app_web_views},
    },
    downloads::DownloadController,
    drafts::DraftController,
    platform::xdg::XdgDirectories,
    profiles::{ProfileId, storage::ProfilePaths},
};
use gtk::prelude::*;
use relm4::{RelmApp, gtk, prelude::*};

struct MadoApp {
    root_widget: gtk::Box,
}

#[relm4::component]
impl SimpleComponent for MadoApp {
    type Init = ProfileId;
    type Input = ();
    type Output = ();

    view! {
        main_window = gtk::ApplicationWindow {
            set_title: Some("Mado"),
            set_child: Some(&model.root_widget),
        }
    }

    fn init(
        profile: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let paths = ProfilePaths::new(&XdgDirectories::discover(), &profile);
        paths
            .prepare()
            .expect("failed to initialize profile state directories");
        let window_state = WindowState::load(&paths.window_state_file()).unwrap_or_else(|error| {
            eprintln!("failed to load window state: {error}");
            WindowState::default()
        });

        let drafts = DraftController::new(paths.draft_state_file());
        let downloads = DownloadController::new();
        let web_view = chatgpt_web_view(&paths, &drafts, &downloads);
        drafts.attach_web_view(&web_view);
        let browser_widget = wrap_with_recovery(&web_view);
        browser_widget.set_vexpand(true);

        let root_widget = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root_widget.append(&browser_widget);
        root_widget.append(&drafts.widget());
        root_widget.append(&downloads.widget());

        let model = MadoApp { root_widget };
        let widgets = view_output!();

        root.set_default_size(window_state.width, window_state.height);
        if window_state.maximized {
            root.maximize();
        }

        let window_state_file = paths.window_state_file();
        root.connect_close_request(move |window| {
            if let Err(error) = WindowState::capture(window).save(&window_state_file) {
                eprintln!("failed to save window state: {error}");
            }
            stop_app_web_views(&relm4::main_application());
            gtk::glib::Propagation::Proceed
        });

        ComponentParts { model, widgets }
    }
}

pub fn run(profile: ProfileId) {
    let app = RelmApp::new("io.github.Millesant.Mado");
    app.allow_multiple_instances(false);

    relm4::main_application().connect_activate(|application| {
        if let Some(window) = application.active_window() {
            window.present();
        }
    });

    app.run::<MadoApp>(profile);
}
