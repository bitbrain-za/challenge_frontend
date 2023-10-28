use crate::components::password;
use gloo_net::http;
use poll_promise::Promise;
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

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct LoginApp {
    logged_in: bool,
    username: String,
    token: Option<String>,
    submit: Option<AuthRequest>,
    #[serde(skip)]
    promise: Option<Promise<LoginResponse>>,
    #[serde(skip)]
    url: String,
    #[serde(skip)]
    login: LoginSchema,
}

impl Default for LoginApp {
    fn default() -> Self {
        Self {
            logged_in: false,
            promise: Default::default(),
            url: option_env!("BACKEND_URL")
                .unwrap_or("http://123.4.5.6:3000/api/auth")
                .to_string(),
            login: LoginSchema {
                email: "admin@admin.com".to_string(),
                password: "password123".to_string(),
            },
            token: None,
            username: "".to_string(),
            submit: None,
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

            log::debug!("Response: {:?}", response);
            let headers = response.headers();
            log::debug!("Headers: {:?}", headers);
            for (key, value) in headers.entries() {
                log::debug!("{}: {:?}", key, value);
            }
            // let cookies = headers.get("set-cookie").unwrap().to_string();
            // log::debug!("Cookies: {:?}", cookies);

            let result: LoginResponse = response.json().await.unwrap();
            ctx.request_repaint(); // wake up UI thread
            log::info!("Result: {:?}", result);
            result
        });

        self.promise = Some(promise);
        self.submit = None;
    }
}

impl super::App for LoginApp {
    fn name(&self) -> &'static str {
        "🔐 Login"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        if let Some(s) = &self.submit {
            match s {
                AuthRequest::Login => {
                    log::debug!("Submitting login request");
                    self.submit_login(ctx);
                }
                AuthRequest::Logout => {
                    log::debug!("Submitting logout request");
                    self.logged_in = false;
                    // self.submit_logout(ctx);
                }
                AuthRequest::Register => {
                    log::debug!("Submitting register request");
                    todo!();
                    // self.submit_register(ctx);
                }
            }
        }
        use super::View as _;
        egui::Window::new(self.name())
            .open(open)
            .default_height(500.0)
            .show(ctx, |ui| self.ui(ui));
    }
}

impl super::View for LoginApp {
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("login_grid")
            .num_columns(2)
            .spacing([20.0, 4.0])
            .striped(false)
            .show(ui, |ui| {
                ui.label("Email:");
                ui.text_edit_singleline(&mut self.login.email);
                ui.end_row();

                ui.label("Password:");
                ui.add(password::password(&mut self.login.password));
                ui.end_row();
            });

        ui.separator();
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                if !self.logged_in {
                    if ui.button("Login").clicked() {
                        self.submit = Some(AuthRequest::Login);
                    }
                } else if ui.button("Logout").clicked() {
                    self.submit = Some(AuthRequest::Logout);
                }
            });
            ui.separator();
            ui.vertical(|ui| {
                if let Some(promise) = &self.promise {
                    if let Some(result) = promise.ready() {
                        match result {
                            LoginResponse::Success {
                                status,
                                access_token,
                            } => {
                                ui.label(format!("status: {}", status));
                                ui.label(format!("token: {}", access_token));
                            }
                            LoginResponse::Failure { status, message } => {
                                ui.label(format!("status: {}", status));
                                ui.label(format!("message: {}", message));
                            }
                        }
                    } else {
                        ui.label("Running...");
                    }
                }
            });
        });
    }
}
