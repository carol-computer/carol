use anyhow::{anyhow, Context};
pub use carol_core::BinaryId;
pub use carol_http::api::{BinaryCreated, MachineCreated};

pub struct Client {
    pub base: String,
    http_client: reqwest::blocking::Client,
}

impl Client {
    pub fn new(base: String) -> Self {
        Self {
            base,
            http_client: reqwest::blocking::Client::new(), // blocking::get() does builder().build()?
        }
    }

    pub fn upload_binary<B: Into<reqwest::blocking::Body>>(
        &self,
        binary_id: &BinaryId,
        binary: B,
    ) -> anyhow::Result<BinaryCreated> {
        let http_response = self
            .post("binaries") // TODO idempotent HTTP PUT?
            .body(binary)
            .send()
            .context("Uploading compiled WASM")?;

        let api_response: BinaryCreated = self
            .decode_response(http_response)
            .context("Parse response")?;

        if api_response.id != *binary_id {
            return Err(anyhow!(
                "Locally computed binary ID {} doesn't match server reported value {}",
                binary_id,
                api_response.id,
            ));
        }

        Ok(api_response)
    }

    pub fn create_machine(&self, binary_id: &BinaryId) -> anyhow::Result<MachineCreated> {
        let http_response = self
            .post(&format!("binaries/{binary_id}"))
            .send()
            .context(format!("Creating machine from {binary_id}"))?;

        self.decode_response::<MachineCreated>(http_response)
            .context("Parse response")
    }

    fn post(&self, path: &str) -> reqwest::blocking::RequestBuilder {
        self.http_client
            .post(format!("{}/{}", self.base, path))
            .header(reqwest::header::ACCEPT, "application/json")
    }

    fn decode_response<B>(&self, response: reqwest::blocking::Response) -> anyhow::Result<B>
    where
        B: for<'de> serde::Deserialize<'de>,
    {
        let response = response.error_for_status()?;

        // FIXME error if no content type specified?
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .map(|h| h.to_str().unwrap_or(""))
            .unwrap_or("");

        if !content_type.is_empty() && content_type != "application/json" {
            return Err(anyhow!("Unsupported response content-type {content_type}"));
        }

        let body = response.bytes().context("Reading server response")?;
        serde_json::from_slice(&body).context("Decoding response body")
    }
}
