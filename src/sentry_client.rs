use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{header, Client, ClientBuilder, StatusCode};
use serde::Deserialize;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone)]
pub struct SentryClient {
    upstream_url: String,
    client: Client,
}

impl Default for SentryClient {
    fn default() -> Self {
        Self {
            upstream_url: String::from("http://web:9000"),
            client: Client::new(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SentryError {
    StatusCodeNotSuccessful(u16),
    RequestError(String),
    ResponseError(String),
    JsonParsingError(String),
}

impl Display for SentryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SentryError::StatusCodeNotSuccessful(status_code) => {
                write!(f, "Status code not successful: {}", status_code)
            }
            SentryError::RequestError(error) => write!(f, "Request error: {}", error),
            SentryError::ResponseError(error) => write!(f, "Response error: {}", error),
            SentryError::JsonParsingError(error) => write!(f, "Json parsing error: {}", error),
        }
    }
}

impl Error for SentryError {}

#[derive(Deserialize)]
struct ListOrganizationResponse {
    pub slug: String,
}

#[derive(Deserialize)]
struct ListProjectResponse {
    pub slug: String,
    pub organization: ListOrganizationResponse,
}

#[derive(Deserialize)]
struct ListProjectClientKeysResponse {
    pub public: String,
}

impl SentryClient {
    pub fn new(upstream_url: String, auth_token: String) -> Self {
        let mut headers = HeaderMap::new();

        if !auth_token.is_empty() {
            let mut auth_value = HeaderValue::from_str(&format!("Bearer {}", auth_token)).unwrap();
            auth_value.set_sensitive(true);
            headers.insert(header::AUTHORIZATION, auth_value);
        }

        let client = ClientBuilder::new()
            .default_headers(headers)
            .build()
            .unwrap();

        Self {
            upstream_url,
            client,
        }
    }
    pub async fn list_organization(&self) -> Result<Vec<String>, SentryError> {
        let url = format!("{}/api/0/organizations/", self.upstream_url);

        let response = self.client.get(&url).send().await;

        match response {
            Ok(response) => {
                if response.status().is_success() {
                    let organization_response =
                        response.json::<Vec<ListOrganizationResponse>>().await;
                    match organization_response {
                        Ok(organizations) => {
                            Ok(organizations.iter().map(|org| org.slug.clone()).collect())
                        }
                        Err(error) => Err(SentryError::JsonParsingError(error.to_string())),
                    }
                } else {
                    Err(SentryError::StatusCodeNotSuccessful(
                        response.status().as_u16(),
                    ))
                }
            }
            Err(error) => Err(SentryError::RequestError(error.to_string())),
        }
    }

    pub async fn list_projects(&self) -> Result<Vec<(String, String)>, SentryError> {
        let url = format!("{}/api/0/projects/", self.upstream_url);

        let response = self.client.get(&url).send().await;

        match response {
            Ok(response) => {
                if response.status().is_success() {
                    let project_response = response.json::<Vec<ListProjectResponse>>().await;
                    match project_response {
                        Ok(projects) => {
                            Ok(projects.iter().map(|project| (project.organization.slug.clone(), project.slug.clone())).collect())
                        }
                        Err(error) => Err(SentryError::JsonParsingError(error.to_string())),
                    }
                } else {
                    Err(SentryError::StatusCodeNotSuccessful(
                        response.status().as_u16(),
                    ))
                }
            }
            Err(error) => Err(SentryError::RequestError(error.to_string())),
        }
    }
    pub async fn list_projects_by_organization_id(&self, organization_id: String) -> Result<Vec<String>, SentryError> {
        let url = format!("{}/api/0/organizations/{}/projects/", self.upstream_url, organization_id);

        let response = self.client.get(&url).send().await;

        match response {
            Ok(response) => {
                if response.status().is_success() {
                    let project_response = response.json::<Vec<ListProjectResponse>>().await;
                    match project_response {
                        Ok(projects) => {
                            Ok(projects.iter().map(|project| project.slug.clone()).collect())
                        }
                        Err(error) => Err(SentryError::JsonParsingError(error.to_string())),
                    }
                } else {
                    Err(SentryError::StatusCodeNotSuccessful(
                        response.status().as_u16(),
                    ))
                }
            }
            Err(error) => Err(SentryError::RequestError(error.to_string())),
        }
    }

    pub async fn list_project_client_keys(
        &self,
        organization_id: String,
        project_id: String,
    ) -> Result<Vec<String>, SentryError> {
        let url = format!(
            "{}/api/0/projects/{}/{}/keys/",
            self.upstream_url, organization_id, project_id
        );

        let response = self.client.get(&url).send().await;

        match response {
            Ok(response) => {
                if response.status().is_success() {
                    let client_keys_response =
                        response.json::<Vec<ListProjectClientKeysResponse>>().await;
                    match client_keys_response {
                        Ok(client_keys_response) => Ok(client_keys_response
                            .iter()
                            .map(|key| key.public.clone())
                            .collect()),
                        Err(error) => Err(SentryError::JsonParsingError(error.to_string())),
                    }
                } else {
                    if response.status() == StatusCode::NOT_FOUND {
                        return Ok(vec![]);
                    }

                    Err(SentryError::StatusCodeNotSuccessful(
                        response.status().as_u16(),
                    ))
                }
            }
            Err(error) => Err(SentryError::RequestError(error.to_string())),
        }
    }
}
