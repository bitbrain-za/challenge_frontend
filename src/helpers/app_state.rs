pub struct AppState {
    pub counter: usize,
    pub logged_in: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            counter: 1,
            logged_in: false,
        }
    }
}
