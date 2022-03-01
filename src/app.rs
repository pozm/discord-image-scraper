use std::future::Future;
use eframe::egui::TextEdit;
use eframe::{egui, epi};
use poll_promise::Promise;
use serenity::{Client, FutureExt,async_trait};
use serenity::client::EventHandler;
use short_crypt::ShortCrypt;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    // this how you opt-out of serialization of a member
    #[cfg_attr(feature = "persistence", serde(skip))]
    encryption : ShortCrypt,
    value: f32,
    show_settings: bool,
    #[cfg_attr(feature = "persistence", serde(skip))]
    discord_token: String,
    stored_discord_token: String,
    download_to: String,
    #[cfg_attr(feature = "persistence", serde(skip))]
    discord_client:Option<Promise<Client>>
}

impl Default for TemplateApp {
    fn default() -> Self {
        let encrypt_key = "poggers";
        Self {
            encryption: ShortCrypt::new(encrypt_key),


            label: "Hello World!".to_owned(),
            value: 2.7,
            show_settings: false,
            discord_token: String::new(),
            stored_discord_token: String::new(),
            download_to: std::env::current_dir()
                .unwrap()
                .display()
                .to_string()
                .to_owned(),
            discord_client:None
        }
    }
}

impl epi::App for TemplateApp {
    fn name(&self) -> &str {
        "eframe template"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::Context,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        println!("setup!!");
        println!("{}", cfg!(feature = "persistence"));
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            println!("loading state");
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default();
            self.discord_token = String::from_utf8((*self.encryption.decrypt_qr_code_alphanumeric(&*self.stored_discord_token).unwrap_or_default()).to_owned()).unwrap()
        }
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        self.stored_discord_token = self.encryption.encrypt_to_qr_code_alphanumeric(&self.discord_token);
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        let Self {
            label,
            value,
            show_settings,
            discord_token,
            download_to,
            encryption,
            stored_discord_token,
            discord_client
        } = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
                ui.menu_button("Settings", |ui| {
                    if ui.button("Toggle").clicked() {
                        *show_settings = !*show_settings;
                        ui.close_menu()
                    }
                })
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(label);
            });

            ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                *value += 1.0;
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/eframe");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.close_menu();
            if discord_client.is_none() {
                if ui.button("Login to discord").clicked() {
                    discord_client.get_or_insert_with(|| {
                        let (sender, prom) = Promise::new();
                        let clientb = Client::builder(&discord_token).event_handler(Handler);
                        clientb.then(|client| async move {
                            println!("log in poggers");
                            sender.send(client.expect("bruh"))
                        }).;
                        prom
                    });
                };

            } else {

            }
            egui::warn_if_debug_build(ui);
        });

        // if *show_settings {
        egui::Window::new("Settings")
            .open(show_settings)
            .show(ctx, |ui| {
                ui.heading("Token");
                ui.add(TextEdit::singleline(discord_token).hint_text("h"));
                ui.heading("Download to");
                if ui.button(format!("select [{}]", *download_to)).clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        *download_to = path.display().to_string();
                    }
                }
            });
        // }
    }
}
