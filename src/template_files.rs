use serde::Serialize;

#[derive(Clone)]
pub struct TemplateFiles {
    pub js_sdk_loader: String,
    pub js_sdk_min_loader: String,
}

#[derive(Serialize)]
struct SentryJavascriptConfig {
    pub dsn: String,
    #[serde(rename = "tracesSampleRate")]
    pub traces_sample_rate: i8,
    #[serde(rename = "replaysSessionSampleRate")]
    pub replays_session_sample_rate: f32,
    #[serde(rename = "replaysOnErrorSampleRate")]
    pub replays_on_error_sample_rate: i8,
}
impl TemplateFiles {
    pub fn build(&self, public_key: String, js_sdk_url: String, dsn: String) -> String {
        let sentry_config = serde_json::to_string(&SentryJavascriptConfig {
            dsn,
            traces_sample_rate: 1,
            replays_session_sample_rate: 0.1,
            replays_on_error_sample_rate: 1,
        })
        .unwrap();

        self.js_sdk_loader
            .clone()
            .replace("{{ publicKey|safe }}", public_key.as_str())
            .replace("{{ jsSdkUrl|safe }}", js_sdk_url.as_str())
            .replace("{{ config|to_json|safe }}", sentry_config.as_str())
            .replace("{{ isLazy|safe|lower }}", "false")
            .replace("{% load sentry_helpers %}", "")
    }

    pub fn build_minified(&self, public_key: String, js_sdk_url: String, dsn: String) -> String {
        let sentry_config = serde_json::to_string(&SentryJavascriptConfig {
            dsn,
            traces_sample_rate: 1,
            replays_session_sample_rate: 0.1,
            replays_on_error_sample_rate: 1,
        })
        .unwrap();

        self.js_sdk_min_loader
            .clone()
            .replace("{{ publicKey|safe }}", public_key.as_str())
            .replace("{{ jsSdkUrl|safe }}", js_sdk_url.as_str())
            .replace("{{ config|to_json|safe }}", sentry_config.as_str())
            .replace("{{ isLazy|safe|lower }}", "false")
            .replace("{% load sentry_helpers %}", "")
    }
}
