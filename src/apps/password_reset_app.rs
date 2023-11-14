use crate::helpers::{
    fetchers::{RequestStatus, Requestor},
    AppState,
};
use egui_notify::Toasts;
use email_address::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Default, Clone, PartialEq, serde::Serialize)]
struct ChangePasswordSchema {
    email: String,
    old_password: String,
    new_password: String,
    #[serde(skip)]
    confirm_password: String,
}

#[derive(Default, Clone, PartialEq, serde::Serialize)]
struct ResetPasswordSchema {
    password: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PasswordResetApp {
    #[serde(skip)]
    email: String,
    #[serde(skip)]
    token: String,
    #[serde(skip)]
    url: String,
    #[serde(skip)]
    toasts: Toasts,
    #[serde(skip)]
    new_password: String,
    #[serde(skip)]
    confirm_password: String,
    #[serde(skip)]
    requestor: Option<Requestor>,
    #[serde(skip)]
    app_state: Arc<Mutex<AppState>>,
}

impl Default for PasswordResetApp {
    fn default() -> Self {
        let url = option_env!("BACKEND_URL")
            .unwrap_or("http://12.34.56.78:9999/")
            .to_string();
        Self {
            requestor: Default::default(),
            url,
            token: "".to_string(),
            email: "".to_string(),
            toasts: Toasts::default(),
            new_password: "".to_string(),
            confirm_password: "".to_string(),
            app_state: Default::default(),
        }
    }
}

impl PasswordResetApp {
    fn submit_reset(&mut self) {
        let submission = ResetPasswordSchema {
            password: self.new_password.clone(),
        };

        let submission = Some(serde_json::to_string(&submission).unwrap());
        let url = format!("{}api/auth/resetpassword/{}", self.url, self.token);
        let app_state = Arc::clone(&self.app_state);
        let mut req = Requestor::new_post(app_state, &url, true, submission);
        req.send();
        self.requestor = Some(req);
    }

    fn check_reset_promise(&mut self) {
        let getter = &mut self.requestor;

        if let Some(getter) = getter {
            let result = &getter.check_promise();

            match result {
                RequestStatus::Failed(message) => {
                    self.toasts
                        .error(format!("Error Resetting password: {}", message))
                        .set_duration(Some(Duration::from_secs(5)));
                }
                RequestStatus::Success(_) => {
                    self.toasts
                        .info("Password reset successfully! Please login.")
                        .set_duration(Some(Duration::from_secs(5)));
                }
                _ => {}
            }
        }
    }

    fn ui_reset(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("login_grid")
            .num_columns(2)
            .spacing([20.0, 4.0])
            .striped(false)
            .show(ui, |ui| {
                ui.label("Email:");
                ui.add(egui::widgets::text_edit::TextEdit::singleline(
                    &mut self.email,
                ));
                ui.end_row();

                ui.label("Token");
                ui.add(egui::widgets::text_edit::TextEdit::singleline(
                    &mut self.token,
                ));
                ui.end_row();

                ui.label("Password:");
                ui.add(
                    egui::widgets::text_edit::TextEdit::singleline(&mut self.new_password)
                        .password(true),
                );
                ui.end_row();

                ui.label("Confirm Password:");
                ui.add(
                    egui::widgets::text_edit::TextEdit::singleline(&mut self.confirm_password)
                        .password(true),
                );
                ui.end_row();
            });

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Submit").clicked() {
                if self.new_password != self.confirm_password {
                    self.toasts
                        .error("Passwords do not match")
                        .set_duration(Some(Duration::from_secs(5)));
                } else if !EmailAddress::is_valid(&self.email) {
                    self.toasts
                        .error("Invalid email address")
                        .set_duration(Some(Duration::from_secs(5)));
                } else {
                    self.submit_reset();
                }
            }
        });
    }
}

impl super::App for PasswordResetApp {
    fn name(&self) -> &'static str {
        "🔐 Password Reset"
    }

    fn set_app_state_ref(&mut self, app_state: Arc<Mutex<AppState>>) {
        self.app_state = app_state;
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        self.check_reset_promise();

        use super::View as _;
        egui::Window::new(self.name())
            .open(open)
            .default_height(500.0)
            .show(ctx, |ui| self.ui(ui));

        self.toasts.show(ctx);
    }
}

impl super::View for PasswordResetApp {
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.ui_reset(ui);
    }
}
