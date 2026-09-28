use gtk::prelude::*;
use relm4::{RelmApp, gtk, prelude::*};
use webkit6::WebView;

use crate::{
    browser::window::{chatgpt_web_view, stop_app_web_views},
    platform::xdg::XdgDirectories,
    profiles::{ProfileId, storage::ProfilePaths},
};

struct MadoApp {
    web_view: WebView,
}

#[relm4::component]
impl SimpleComponent for MadoApp {
    type Init = ProfileId;
    type Input = ();
    type Output = ();

    view! {
        main_window = gtk::ApplicationWindow {
            set_title: Some("Mado"),
            set_default_size: (1280, 900),
            set_child: Some(&model.web_view),
        }
    }

    fn init(
        profile: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let paths = ProfilePaths::new(&XdgDirectories::discover(), &profile);
        let model = MadoApp {
            web_view: chatgpt_web_view(&paths),
        };
        root.connect_close_request(move |_| {
            stop_app_web_views(&relm4::main_application());
            gtk::glib::Propagation::Proceed
        });
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

pub fn run(profile: ProfileId) {
    RelmApp::new("io.github.Millesant.Mado").run::<MadoApp>(profile);
}
