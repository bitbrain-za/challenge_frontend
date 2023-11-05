use crate::helpers::refresh;
use egui_notify::Toasts;
use gloo_net::http;
use poll_promise::Promise;
use std::time::Duration;
use web_sys::RequestCredentials;

#[derive(serde::Deserialize, serde::Serialize)]
enum AuthRequest {
    Login,
    Logout,
    Register,
}

#[derive(Clone, PartialEq, serde::Serialize)]
struct LoginSchema {
    email: String,
    password: String,
}

#[derive(Default, Clone, PartialEq, serde::Serialize)]
struct RegisterSchema {
    name: String,
    email: String,
    password: String,
    #[serde(skip)]
    confirm_password: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct RegisterResponse {
    status: String,
    message: String,
}

impl RegisterResponse {
    fn is_success(&self) -> bool {
        self.status.to_lowercase() == "success"
    }

    async fn from_response(response: http::Response) -> Self {
        response.json::<RegisterResponse>().await.unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum LoginResponse {
    Success {
        status: String,
        access_token: String,
    },
    Failure {
        status: String,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
enum LoginState {
    LoggedIn(String),
    LoggedOut,
    RegisterNewUser,
    Expired,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct LoginApp {
    username: String,
    token: Option<String>,
    submit: Option<AuthRequest>,
    #[serde(skip)]
    login_promise: Option<Promise<Result<LoginState, String>>>,
    #[serde(skip)]
    register_promise: Option<Promise<RegisterResponse>>,
    #[serde(skip)]
    url: String,
    #[serde(skip)]
    login: LoginSchema,
    #[serde(skip)]
    state: LoginState,
    #[serde(skip)]
    register: RegisterSchema,
    #[serde(skip)]
    toasts: Toasts,
    #[serde(skip)]
    token_refresh_promise: refresh::RefreshPromise,
}

impl Default for LoginApp {
    fn default() -> Self {
        let url = option_env!("BACKEND_URL")
            .unwrap_or("http://localhost:3000/")
            .to_string();
        Self {
            token_refresh_promise: refresh::submit_refresh(&url),
            login_promise: Default::default(),
            register_promise: Default::default(),
            url,
            login: LoginSchema {
                email: "admin@admin.com".to_string(),
                password: "password123".to_string(),
            },
            token: None,
            username: "".to_string(),
            submit: None,
            state: LoginState::LoggedOut,
            register: RegisterSchema::default(),
            toasts: Toasts::default(),
        }
    }
}

impl LoginApp {
    fn submit_login(&mut self, ctx: &egui::Context) {
        let submission = self.login.clone();

        let url = format!("{}api/auth/login", self.url);
        log::debug!("Sending to {}", url);
        let ctx = ctx.clone();

        let promise = Promise::spawn_local(async move {
            let response = http::Request::post(&url)
                .credentials(RequestCredentials::Include)
                .json(&submission)
                .unwrap()
                .send()
                .await
                .unwrap();
            let result: LoginResponse = response.json().await.unwrap();

            let result = match result {
                LoginResponse::Success { .. } => Ok(LoginState::LoggedIn(submission.email)),
                LoginResponse::Failure { status: _, message } => {
                    log::error!("Failed to login: {}", message);
                    Err(message)
                }
            };

            ctx.request_repaint(); // wake up UI thread
            result
        });

        self.login_promise = Some(promise);
        self.submit = None;
    }
    fn submit_logout(&mut self, ctx: &egui::Context) {
        let url = format!("{}api/auth/logout", self.url);
        log::debug!("Sending to {}", url);
        let ctx = ctx.clone();

        let promise = Promise::spawn_local(async move {
            let response = http::Request::post(&url)
                .credentials(RequestCredentials::Include)
                .send()
                .await
                .unwrap();
            let result = match response.status() {
                200 => Ok(LoginState::LoggedOut),
                401 => {
                    let text = response.text().await;
                    let text = text.map(|text| text.to_owned());
                    let text = match text {
                        Ok(text) => text,
                        Err(e) => e.to_string(),
                    };
                    log::warn!("Auth Error: {:?}", text);
                    Ok(LoginState::Expired)
                }
                _ => {
                    let text = response.text().await.unwrap();
                    Err(text)
                }
            };
            ctx.request_repaint(); // wake up UI thread
            result
        });

        self.login_promise = Some(promise);
        self.submit = None;
    }
    fn submit_register(&mut self, ctx: &egui::Context) {
        if self.register == RegisterSchema::default() {
            return;
        }
        let submission = self.register.clone();
        let url = format!("{}api/auth/register", self.url);
        let ctx = ctx.clone();

        let promise = Promise::spawn_local(async move {
            let response = http::Request::post(&url)
                .json(&submission)
                .unwrap()
                .send()
                .await
                .unwrap();
            let result = RegisterResponse::from_response(response).await;
            log::info!("Result: {:?}", result);
            ctx.request_repaint(); // wake up UI thread
            result
        });

        self.register_promise = Some(promise);
        self.submit = None;
    }

    fn check_login_promise(&mut self) {
        if let Some(promise) = &self.login_promise {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(LoginState::LoggedIn(email)) => {
                        self.state = LoginState::LoggedIn(email.clone());
                        self.toasts
                            .info(format!("Logged in as {}.", email))
                            .set_duration(Some(Duration::from_secs(5)));
                    }
                    Ok(LoginState::LoggedOut) => {
                        self.state = LoginState::LoggedOut;
                        self.toasts
                            .info("Logged out.")
                            .set_duration(Some(Duration::from_secs(5)));
                    }
                    Ok(LoginState::Expired) => {
                        self.state = LoginState::Expired;
                        self.token_refresh_promise = refresh::submit_refresh(&self.url);
                    }
                    Err(e) => {
                        self.toasts
                            .error(format!("Failed: {}", e))
                            .set_duration(Some(Duration::from_secs(5)));
                        log::error!("Error: {}", e);
                    }
                    _ => {
                        log::error!("How did you get here?!");
                    }
                }
                self.login_promise = None;
            }
        }
    }

    fn check_register_promise(&mut self) {
        if let Some(promise) = &self.register_promise {
            if let Some(result) = promise.ready() {
                if result.is_success() {
                    self.toasts
                        .info("Registered successfully! Please login.")
                        .set_duration(Some(Duration::from_secs(5)));
                } else {
                    self.toasts
                        .error("Failed to register!")
                        .set_duration(Some(Duration::from_secs(5)));
                }
                self.register_promise = None;
                self.register = RegisterSchema::default();
            }
        }
    }

    fn ui_logged_in(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                if ui.button("Logout").clicked() {
                    self.submit = Some(AuthRequest::Logout);
                }
            });
            ui.separator();
            ui.vertical(|ui| {
                ui.label(format!("Logged in as: {}", self.login.email));
            });
        });
    }

    fn ui_logged_out(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("login_grid")
            .num_columns(2)
            .spacing([20.0, 4.0])
            .striped(false)
            .show(ui, |ui| {
                ui.label("Email:");
                ui.add(egui::widgets::text_edit::TextEdit::singleline(
                    &mut self.login.email,
                ));
                ui.end_row();

                ui.label("Password:");
                ui.add(
                    egui::widgets::text_edit::TextEdit::singleline(&mut self.login.password)
                        .password(true),
                );
                ui.end_row();
            });

        ui.separator();
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                if ui.button("Login").clicked() {
                    self.submit = Some(AuthRequest::Login);
                }
            });
            ui.separator();
            ui.vertical(|ui| {
                if ui.button("Register").clicked() {
                    self.register = RegisterSchema::default();
                    self.state = LoginState::RegisterNewUser;
                }
            });
        });
    }

    fn ui_register(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("login_grid")
            .num_columns(2)
            .spacing([20.0, 4.0])
            .striped(false)
            .show(ui, |ui| {
                ui.label("Name:");
                ui.add(egui::widgets::text_edit::TextEdit::singleline(
                    &mut self.register.name,
                ));
                ui.end_row();

                ui.label("Email:");
                ui.add(egui::widgets::text_edit::TextEdit::singleline(
                    &mut self.register.email,
                ));
                ui.end_row();

                ui.label("Password:");
                ui.add(
                    egui::widgets::text_edit::TextEdit::singleline(&mut self.register.password)
                        .password(true),
                );
                ui.end_row();

                ui.label("Confirm Password:");
                ui.add(
                    egui::widgets::text_edit::TextEdit::singleline(
                        &mut self.register.confirm_password,
                    )
                    .password(true),
                );
                ui.end_row();
            });

        ui.separator();
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                if ui.button("Register").clicked() {
                    if self.register.password != self.register.confirm_password {
                        self.toasts
                            .error("Passwords do not match!")
                            .set_duration(Some(Duration::from_secs(5)));
                    } else {
                        self.submit = Some(AuthRequest::Register);
                    }
                }
            });
            ui.separator();
            ui.vertical(|ui| {
                if ui.button("Cancel").clicked() {
                    self.register = RegisterSchema::default();
                    self.state = LoginState::LoggedOut;
                }
            });
        });
    }
}

impl super::App for LoginApp {
    fn name(&self) -> &'static str {
        "🔐 Login"
    }
    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        match refresh::check_refresh_promise(&mut self.token_refresh_promise) {
            refresh::RefreshStatus::InProgress => {}
            refresh::RefreshStatus::Success => {
                if self.state == LoginState::Expired {
                    self.submit = Some(AuthRequest::Logout);
                }
                self.state = LoginState::LoggedIn(self.login.email.clone());
            }
            refresh::RefreshStatus::Failed(_) => {
                self.state = LoginState::LoggedOut;
                self.submit = None;
            }
            _ => (),
        }
        self.check_login_promise();
        self.check_register_promise();

        if let Some(s) = &self.submit {
            match s {
                AuthRequest::Login => {
                    log::debug!("Submitting login request");
                    self.submit_login(ctx);
                }
                AuthRequest::Logout => {
                    if self.state != LoginState::LoggedOut {
                        log::debug!("Submitting logout request");
                        self.submit_logout(ctx);
                    }
                }
                AuthRequest::Register => {
                    log::debug!("Submitting register request");
                    self.submit_register(ctx);
                }
            }
        }
        use super::View as _;
        egui::Window::new(self.name())
            .open(open)
            .default_height(500.0)
            .show(ctx, |ui| self.ui(ui));

        self.toasts.show(ctx);
    }
}

impl super::View for LoginApp {
    fn ui(&mut self, ui: &mut egui::Ui) {
        match self.state {
            LoginState::LoggedIn(..) => self.ui_logged_in(ui),
            LoginState::LoggedOut => self.ui_logged_out(ui),
            LoginState::RegisterNewUser => self.ui_register(ui),
            LoginState::Expired => {
                ui.label("Your session has expired. Please login again.");
                self.ui_logged_out(ui);
            }
        }
    }
}
