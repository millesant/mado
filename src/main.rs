mod application;
mod browser;
mod downloads;
mod drafts;
mod platform;
mod profiles;
mod web;

fn main() {
    let profile = match profiles::selected_profile() {
        Ok(profile) => profile,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };

    application::run(profile);
}
