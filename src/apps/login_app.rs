use crate::helpers::{
    fetchers::{RequestStatus, Requestor},
    AppState, LoginState,
};
use egui_notify::Toasts;
use email_address::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

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

impl LoginSchema {
    fn to_forgot_password(&self) -> ForgotPasswordSchema {
        ForgotPasswordSchema {
            email: self.email.clone(),
        }
    }
}

#[derive(Clone, PartialEq, serde::Serialize)]
struct ForgotPasswordSchema {
    email: String,
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
enum LoginAppState {
    Login,
    RegisterNewUser,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct LoginApp {
    username: String,
    token: Option<String>,
    #[serde(skip)]
    url: String,
    #[serde(skip)]
    login: LoginSchema,
    #[serde(skip)]
    state: LoginAppState,
    #[serde(skip)]
    register: RegisterSchema,
    #[serde(skip)]
    toasts: Toasts,
    #[serde(skip)]
    login_requestor: Option<Requestor>,
    #[serde(skip)]
    logout_requestor: Option<Requestor>,
    #[serde(skip)]
    register_requestor: Option<Requestor>,
    #[serde(skip)]
    reset_pass_requestor: Option<Requestor>,
    #[serde(skip)]
    app_state: Arc<Mutex<AppState>>,
}

impl Default for LoginApp {
    fn default() -> Self {
        let url = option_env!("BACKEND_URL")
            .unwrap_or("http://12.34.56.78:3000/")
            .to_string();
        Self {
            url,
            login: LoginSchema {
                email: "admin@admin.com".to_string(),
                password: "password123".to_string(),
            },
            token: None,
            username: "".to_string(),
            state: LoginAppState::Login,
            register: RegisterSchema::default(),
            toasts: Toasts::default(),
            app_state: Default::default(),
            login_requestor: None,
            logout_requestor: None,
            register_requestor: None,
            reset_pass_requestor: None,
        }
    }
}

impl LoginApp {
    fn submit_login(&mut self) {
        let submission = Some(serde_json::to_string(&self.login).unwrap());
        let url = format!("{}api/auth/login", self.url);
        let app_state = Arc::clone(&self.app_state);
        let mut req = Requestor::new_post(app_state, &url, true, submission);
        req.send();
        self.login_requestor = Some(req);
    }

    fn submit_logout(&mut self) {
        let url = format!("{}api/auth/logout", self.url);
        let app_state = Arc::clone(&self.app_state);
        let mut req = Requestor::new_post(app_state, &url, true, None);
        req.send();
        self.logout_requestor = Some(req);
    }

    fn submit_register(&mut self) {
        let submission = Some(serde_json::to_string(&self.register).unwrap());
        let url = format!("{}api/auth/register", self.url);
        let app_state = Arc::clone(&self.app_state);
        let mut req = Requestor::new_post(app_state, &url, false, submission);
        req.send();
        self.register_requestor = Some(req);
    }

    fn submit_forgot_password(&mut self) {
        let url = format!("{}api/auth/forgotpassword", self.url);
        let submission = Some(serde_json::to_string(&self.login.to_forgot_password()).unwrap());
        let app_state = Arc::clone(&self.app_state);
        let mut req = Requestor::new_post(app_state, &url, false, submission);
        req.send();
        self.reset_pass_requestor = Some(req);
        self.toasts
            .info(format!(
                "If {} is a registered address you will receive a password reset link shortly.",
                self.login.email
            ))
            .set_duration(Some(Duration::from_secs(5)));
    }

    fn check_login_promise(&mut self) {
        let getter = &mut self.login_requestor;

        if let Some(getter) = getter {
            let result = &getter.check_promise();
            match result {
                RequestStatus::Failed(err) => {
                    self.toasts
                        .error(format!("Failed: {}", err))
                        .set_duration(Some(Duration::from_secs(5)));

                    log::error!("Error sending: {}", err);
                    self.login_requestor = None;
                }
                RequestStatus::Success(text) => {
                    log::debug!("Success: {}", text);
                    let result: LoginResponse = serde_json::from_str(text).unwrap();
                    match result {
                        LoginResponse::Success { .. } => {
                            self.toasts
                                .info(format!("Logged in: {}", &self.login.email))
                                .set_duration(Some(Duration::from_secs(5)));

                            AppState::set_logged_in(&self.app_state);
                        }
                        LoginResponse::Failure { status: _, message } => {
                            log::error!("Failed to login: {}", message);
                            self.toasts
                                .error(format!("Failed to login: {}", message))
                                .set_duration(Some(Duration::from_secs(5)));
                        }
                    };
                    self.login_requestor = None;
                }
                _ => {}
            }
        }
    }

    fn check_logout_promise(&mut self) {
        let getter = &mut self.logout_requestor;

        if let Some(getter) = getter {
            let result = &getter.check_promise();
            match result {
                RequestStatus::Failed(err) => {
                    self.toasts
                        .error(format!("Failed: {}", err))
                        .set_duration(Some(Duration::from_secs(5)));

                    log::error!("Error sending: {}", err);
                    self.logout_requestor = None;
                }
                RequestStatus::Success(text) => {
                    log::debug!("Success: {}", text);
                    self.toasts
                        .info(format!("Logged out: {}", &self.login.email))
                        .set_duration(Some(Duration::from_secs(5)));
                    AppState::set_logged_out(&self.app_state);
                    self.logout_requestor = None;
                }
                _ => {}
            }
        }
    }

    fn check_register_promise(&mut self) {
        let getter = &mut self.register_requestor;

        if let Some(getter) = getter {
            let result = &getter.check_promise();
            match result {
                RequestStatus::Failed(err) => {
                    self.register_requestor = None;
                    self.toasts
                        .error(format!("Failed: {}", err))
                        .set_duration(Some(Duration::from_secs(5)));

                    log::error!("Error sending: {}", err);
                }
                RequestStatus::Success(text) => {
                    log::debug!("Success: {}", text);
                    let result: RegisterResponse = serde_json::from_str(text).unwrap();
                    if result.is_success() {
                        self.toasts
                            .info("Registered successfully! Please login.")
                            .set_duration(Some(Duration::from_secs(5)));
                        self.toasts
                            .info("Please check your junk folder for the registration email.")
                            .set_duration(Some(Duration::from_secs(5)));
                    } else {
                        self.toasts
                            .error("Failed to register!")
                            .set_duration(Some(Duration::from_secs(5)));
                    }
                    self.register_requestor = None;
                }
                _ => {}
            }
        }
    }

    fn check_reset_password_promise(&mut self) {
        let getter = &mut self.reset_pass_requestor;

        if let Some(getter) = getter {
            let result = &getter.check_promise();
            match result {
                RequestStatus::Failed(err) => {
                    self.toasts
                        .error(format!("Failed: {}", err))
                        .set_duration(Some(Duration::from_secs(5)));

                    log::error!("Error sending: {}", err);
                }
                RequestStatus::Success(text) => {
                    log::debug!("Success: {}", text);
                    let result: RegisterResponse = serde_json::from_str(text).unwrap();
                    if result.is_success() {
                        self.toasts
                            .info("Password reset token sent")
                            .set_duration(Some(Duration::from_secs(5)));
                        self.toasts
                            .info("Please check your junk folder for the registration email.")
                            .set_duration(Some(Duration::from_secs(5)));
                    }
                    self.reset_pass_requestor = None;
                }
                _ => {}
            }
        }
    }

    fn ui_logged_in(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                if ui.button("Logout").clicked() {
                    self.submit_logout()
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
                    self.submit_login();
                }
            });
            ui.vertical(|ui| {
                if ui.button("Forgot Password").clicked() {
                    self.submit_forgot_password();
                }
            });
            ui.separator();
            ui.vertical(|ui| {
                if ui.button("Register").clicked() {
                    self.register = RegisterSchema::default();
                    self.state = LoginAppState::RegisterNewUser;
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
                            .error("Passwords do not match")
                            .set_duration(Some(Duration::from_secs(5)));
                    } else if !EmailAddress::is_valid(&self.register.email) {
                        self.toasts
                            .error("Invalid email address")
                            .set_duration(Some(Duration::from_secs(5)));
                    } else if !self
                        .register
                        .email
                        .contains(option_env!("ALLOWED_DOMAIN").unwrap_or("dummy.com"))
                    {
                        self.toasts
                            .error(format!(
                                "Email must be from {}",
                                option_env!("ALLOWED_DOMAIN").unwrap_or("dummy.com")
                            ))
                            .set_duration(Some(Duration::from_secs(5)));
                    } else {
                        self.submit_register();
                    }
                }
            });
            ui.separator();
            ui.vertical(|ui| {
                if ui.button("Cancel").clicked() {
                    self.register = RegisterSchema::default();
                    self.state = LoginAppState::Login;
                }
            });
        });
    }
}

impl super::App for LoginApp {
    fn name(&self) -> &'static str {
        "🔐 Login"
    }

    fn set_app_state_ref(&mut self, app_state: Arc<Mutex<AppState>>) {
        self.app_state = app_state;
        let app_state = Arc::clone(&self.app_state);
        let mut req = Requestor::new_refresh(app_state);
        req.send();
        self.login_requestor = Some(req);
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        self.check_login_promise();
        self.check_logout_promise();
        self.check_register_promise();
        self.check_reset_password_promise();

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
        let app = Arc::clone(&self.app_state);
        let app = app.lock().unwrap();
        let logged_in = app.logged_in.clone();

        match self.state {
            LoginAppState::Login => match logged_in {
                LoginState::LoggedIn => self.ui_logged_in(ui),
                LoginState::LoggedOut => self.ui_logged_out(ui),
            },
            LoginAppState::RegisterNewUser => self.ui_register(ui),
        }
    }
}
