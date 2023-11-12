use crate::helpers::AppState;
use egui_notify::Toasts;
use email_address::*;
use gloo_net::http;
use poll_promise::Promise;
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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct Response {
    status: String,
    message: String,
}

impl Response {
    fn is_success(&self) -> bool {
        self.status.to_lowercase() == "success"
    }

    async fn from_response(response: http::Response) -> Self {
        response.json::<Response>().await.unwrap()
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PasswordResetApp {
    #[serde(skip)]
    email: String,
    #[serde(skip)]
    token: String,
    #[serde(skip)]
    promise: Option<Promise<Response>>,
    #[serde(skip)]
    url: String,
    #[serde(skip)]
    toasts: Toasts,
    #[serde(skip)]
    new_password: String,
    #[serde(skip)]
    confirm_password: String,
    #[serde(skip)]
    app_state: Arc<Mutex<AppState>>,
}

impl Default for PasswordResetApp {
    fn default() -> Self {
        let url = option_env!("BACKEND_URL")
            .unwrap_or("http://12.34.56.78:9999/")
            .to_string();
        Self {
            promise: Default::default(),
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

        let url = format!("{}api/auth/resetpassword/{}", self.url, self.token);

        let promise = Promise::spawn_local(async move {
            let response = http::Request::post(&url)
                .json(&submission)
                .unwrap()
                .send()
                .await
                .unwrap();
            let result = Response::from_response(response).await;
            log::info!("Result: {:?}", result);
            result
        });

        self.promise = Some(promise);
    }

    fn check_reset_promise(&mut self) {
        if let Some(promise) = &self.promise {
            if let Some(result) = promise.ready() {
                if result.is_success() {
                    self.toasts
                        .info("Password reset successfully! Please login.")
                        .set_duration(Some(Duration::from_secs(5)));
                } else {
                    self.toasts
                        .error(format!("Error Resetting password: {}", result.message))
                        .set_duration(Some(Duration::from_secs(5)));
                }
                self.promise = None;
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
