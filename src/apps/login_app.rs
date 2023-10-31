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

#[derive(Clone, PartialEq, serde::Serialize)]
struct RegisterSchema {
    name: String,
    email: String,
    password: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct RegisterResponse {
    status: String,
    message: String,
}

impl RegisterResponse {
    fn _is_success(&self) -> bool {
        self.status.to_lowercase() == "success"
    }

    fn _is_failure(&self) -> bool {
        !self._is_success()
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
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct LoginApp {
    username: String,
    token: Option<String>,
    submit: Option<AuthRequest>,
    #[serde(skip)]
    promise: Option<Promise<Result<LoginState, String>>>,
    #[serde(skip)]
    register_promise: Option<Promise<RegisterResponse>>,
    #[serde(skip)]
    url: String,
    #[serde(skip)]
    login: LoginSchema,
    #[serde(skip)]
    state: LoginState,
    #[serde(skip)]
    register: Option<RegisterSchema>,
}

impl Default for LoginApp {
    fn default() -> Self {
        Self {
            promise: Default::default(),
            register_promise: Default::default(),
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
            state: LoginState::LoggedOut,
            register: None,
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
            log::info!("Result: {:?}", result);

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

        self.promise = Some(promise);
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
                _ => {
                    let text = response.text().await.unwrap();
                    Err(text)
                }
            };
            ctx.request_repaint(); // wake up UI thread
            result
        });

        self.promise = Some(promise);
        self.submit = None;
    }
    fn submit_register(&mut self, ctx: &egui::Context) {
        if self.register.is_none() {
            return;
        }
        let submission = self.register.clone().unwrap();
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
                    self.submit_logout(ctx);
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
    }
}

impl super::View for LoginApp {
    fn ui(&mut self, ui: &mut egui::Ui) {
        if let Some(promise) = &self.promise {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(LoginState::LoggedIn(email)) => {
                        self.state = LoginState::LoggedIn(email.clone());
                    }
                    Ok(LoginState::LoggedOut) => {
                        self.state = LoginState::LoggedOut;
                    }
                    Err(e) => {
                        log::error!("Error: {}", e);
                    }
                }
            }
        }

        egui::Grid::new("login_grid")
            .num_columns(2)
            .spacing([20.0, 4.0])
            .striped(false)
            .show(ui, |ui| {
                ui.label("Email:");
                ui.add(
                    egui::widgets::text_edit::TextEdit::singleline(&mut self.login.email)
                        .interactive(self.state == LoginState::LoggedOut),
                );
                ui.end_row();

                ui.label("Password:");
                ui.add(password::password(
                    &mut self.login.password,
                    Some(self.state == LoginState::LoggedOut),
                ));
                ui.end_row();
            });

        ui.separator();
        ui.horizontal(|ui| {
            ui.vertical(|ui| match self.state {
                LoginState::LoggedIn(..) => {
                    if ui.button("Logout").clicked() {
                        self.submit = Some(AuthRequest::Logout);
                    }
                }
                LoginState::LoggedOut => {
                    if ui.button("Login").clicked() {
                        self.submit = Some(AuthRequest::Login);
                    }
                }
            });
            ui.separator();
            ui.vertical(|ui| match self.state {
                LoginState::LoggedIn(..) => {
                    ui.label(format!("Logged in as: {}", self.login.email));
                }
                LoginState::LoggedOut => {
                    ui.label("Not logged in");
                }
            });
        });
    }
}
