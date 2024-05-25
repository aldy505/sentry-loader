#[derive(Clone)]
pub struct SentryDsnBuilder {
    pub sentry_hostname: String,
    pub secure: bool,
}

impl Default for SentryDsnBuilder {
    fn default() -> Self {
        Self {
            sentry_hostname: String::from("localhost:9000"),
            secure: false,
        }
    }
}

impl SentryDsnBuilder {
    pub fn build_dsn(&self, public_key: String, project_id: String) -> String {
        format!(
            "{}://{}@{}/{}",
            match self.secure {
                true => "https",
                false => "http",
            },
            public_key,
            self.sentry_hostname,
            project_id
        )
    }
}
