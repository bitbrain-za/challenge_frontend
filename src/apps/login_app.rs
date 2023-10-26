use crate::components::password;
use poll_promise::Promise;

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

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub enum SubmissionResult {
    Success {
        status: String,
        access_token: String,
    },
    Failure {
        message: String,
    },
}

struct SubmissionResponse {
    _response: ehttp::Response,
    result: SubmissionResult,
}

impl SubmissionResponse {
    fn from_response(_: &egui::Context, response: ehttp::Response) -> Self {
        let _ = response.content_type().unwrap_or_default();
        let text = response.text();
        let text = text.map(|text| text.to_owned());
        log::debug!("Response: {:?}", text);
        let result: SubmissionResult = serde_json::from_str(text.as_ref().unwrap()).unwrap();

        Self {
            _response: response,
            result,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct LoginApp {
    logged_in: bool,
    username: String,
    token: Option<String>,
    submit: Option<AuthRequest>,
    #[serde(skip)]
    promise: Option<Promise<ehttp::Result<SubmissionResponse>>>,
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
                email: "".to_string(),
                password: "".to_string(),
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

        let url = format!("{}submit", self.url);
        log::debug!("Sending to {}", url);
        let ctx = ctx.clone();
        let (sender, promise) = Promise::new();

        let submission = serde_json::to_string(&submission).unwrap();
        let request = ehttp::Request::post(url, submission.as_bytes().to_vec());
        ehttp::fetch(request, move |response| {
            ctx.request_repaint(); // wake up UI thread
            let resource =
                response.map(|response| SubmissionResponse::from_response(&ctx, response));
            sender.send(resource);
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
                    // self.submit_login(ctx);
                    self.logged_in = true;
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
                            Ok(submission_response) => match &submission_response.result {
                                SubmissionResult::Success {
                                    status,
                                    access_token,
                                } => {
                                    ui.label(format!("status: {}", status));
                                    ui.label(format!("token: {}", access_token));
                                }
                                SubmissionResult::Failure { message } => {
                                    ui.label(format!("Message: {}", message));
                                }
                            },
                            Err(error) => {
                                log::error!("Failed to fetch scores: {}", error);
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
