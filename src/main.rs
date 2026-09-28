mod application;
mod browser;
mod platform;
mod profiles;

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
