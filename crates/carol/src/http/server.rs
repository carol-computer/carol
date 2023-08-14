use super::api::{self, *};
use crate::config;
use anyhow::{anyhow, Context};
use carol_core::{BinaryId, MachineId};
use carol_host::{GuestError, State};
use hyper::http::uri::PathAndQuery;
use hyper::http::HeaderValue;
use hyper::service::{make_service_fn, service_fn};
use hyper::{body::HttpBody, Body, Method, Request, Response, Server, StatusCode};
use hyper::{header, Uri};
use std::collections::HashMap;
use std::convert::Infallible;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{event, span, Instrument, Level};

#[derive(Debug)]
pub struct Problem {
    host_error: anyhow::Error,
    client_desc: String,
    status: StatusCode,
    extra_headers: Vec<(String, String)>,
    extra_fields: HashMap<String, String>,
}

impl Problem {
    pub fn new(client_desc: String, host_error: anyhow::Error, status: StatusCode) -> Self {
        Self {
            host_error,
            client_desc,
            status,
            extra_headers: vec![],
            extra_fields: Default::default(),
        }
    }

    pub fn guest_error(guest_error: GuestError) -> Self {
        match guest_error {
            GuestError::Panic {
                ref backtrace,
                ref message,
            } => {
                let mut extra_fields = HashMap::default();
                if let Some(bt) = backtrace {
                    extra_fields.insert("backtrace".into(), format!("{}", bt));
                }
                Self {
                    client_desc: message.clone(),
                    status: StatusCode::BAD_REQUEST,
                    extra_headers: vec![],
                    host_error: guest_error.into(),
                    extra_fields,
                }
            }
            GuestError::Other(error) => Self::internal_server_error(error),
        }
    }

    pub fn bad_request(client_desc: String, host_error: anyhow::Error) -> Self {
        Self::new(client_desc, host_error, StatusCode::BAD_REQUEST)
    }

    pub fn internal_server_error(host_error: anyhow::Error) -> Self {
        Self::new(
            "internal server error".into(),
            host_error,
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    }

    pub fn not_found(path: &str) -> Self {
        Self::new(
            format!("{} not found", path),
            anyhow!("resource not found: {}", path),
            StatusCode::NOT_FOUND,
        )
    }

    pub fn method_not_allowed(path: &str, method: &str, allowed: &[&str]) -> Self {
        let mut problem = Self::new(
            format!("HTTP method {} not supported on {}", method, path),
            anyhow!(
                "HTTP method {} called on {} but it's not supported",
                method,
                path
            ),
            StatusCode::METHOD_NOT_ALLOWED,
        );

        problem
            .extra_headers
            .push(("Allow".into(), allowed.join(", ")));
        problem
    }
    pub fn invalid_path_element<T: std::any::Any>(error: anyhow::Error, val: &str) -> Self {
        Self::new(
            format!(
                "path element {} is not a valid {}",
                val,
                std::any::type_name::<T>()
            ),
            error,
            StatusCode::BAD_REQUEST,
        )
    }

    pub fn into_json_body(self) -> Vec<u8> {
        #[derive(serde::Serialize)]
        struct ProblemBody {
            error: String,
            #[serde(flatten)]
            extra_fields: HashMap<String, String>,
        }
        serde_json::to_vec_pretty(&ProblemBody {
            error: self.client_desc,
            extra_fields: self.extra_fields,
        })
        .unwrap()
    }
}

fn build_response<B: api::Response>(app_response: &B) -> Response<Body> {
    let body = serde_json::to_vec_pretty(app_response).unwrap();
    let mut response = Response::new(Body::from(body));
    let headers = response.headers_mut();
    app_response.set_headers(headers);
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str("application/json").unwrap(),
    );
    *response.status_mut() = app_response.status();
    response
}

pub async fn dispatch(mut req: Request<Body>, state: State) -> Result<Response<Body>, Problem> {
    let path = req.uri().path();
    let segments = {
        let mut segments = path.split('/');
        // ignore first `/`
        let _ = segments.next();
        segments.collect::<Vec<_>>()
    };
    match (req.method(), &segments[..]) {
        (&Method::GET, [""]) | (&Method::GET, ["index.html"]) => Ok(Response::new(Body::from(
            b"<html><body><h1> THIS IS A CAROL NODE </h1><p> In the future you will be able to configure this page.</p></body></html>".to_vec(),
        ))),
        (&Method::POST, ["binaries"]) => {
            let body = slurp_request_body(&mut req).await?;
            let binary_id = BinaryId::new(&body);
            let already_exists = state.get_binary(binary_id).is_some();
            let span = span!(
                Level::INFO,
                "POST /binaries",
                binary_id = binary_id.to_string()
            );
            let _enter = span.enter();

            if already_exists {
                event!(Level::DEBUG, "already existing binary ignored");
                let mut response = build_response(&BinaryCreated { id: binary_id });
                *response.status_mut() = StatusCode::OK;
                Ok(response)
            } else {
                let compiled_binary = state
                    .executor()
                    .load_binary_from_wasm_binary(&body)
                    .map_err(|e| {
                        Problem::new(
                            format!("Invalid WASM binary with id {}: {}", binary_id, e),
                            e,
                            StatusCode::BAD_REQUEST,
                        )
                    })?;

                debug_assert_eq!(compiled_binary.binary_id(), binary_id);
                state.insert_binary(compiled_binary);
                event!(Level::INFO, "new binary uploaded");
                Ok(build_response(&BinaryCreated { id: binary_id }))
            }
        }
        (method, ["binaries", binary_id]) => {
            let binary_id = BinaryId::from_str(binary_id)
                .map_err(|e| Problem::invalid_path_element::<BinaryId>(e.into(), binary_id))?;
            let _binary = state
                .get_binary(binary_id)
                .ok_or(Problem::not_found(path))?;

            match method {
                &Method::GET => {
                    let mut response = Response::new(Body::empty());
                    *response.status_mut() = StatusCode::NO_CONTENT;
                    Ok(response)
                }
                &Method::POST => {
                    let params = slurp_request_body(&mut req).await?;
                    let (already_exists, machine_id) = state.insert_machine(binary_id, params);
                    let mut response = build_response(&MachineCreated { id: machine_id });

                    if already_exists {
                        *response.status_mut() = StatusCode::OK;
                    } else {
                        event!(
                            Level::INFO,
                            machine_id = machine_id.to_string(),
                            "machine created"
                        );
                    }
                    Ok(response)
                }
                method => Err(Problem::method_not_allowed(
                    path,
                    method.as_str(),
                    &["POST", "GET"],
                )),
            }
        }
        (method, ["machines", machine_id]) => {
            let machine_id = MachineId::from_str(machine_id)
                .map_err(|e| Problem::invalid_path_element::<MachineId>(e.into(), machine_id))?;
            let (binary_id, params, compiled_binary) = {
                let (binary_id, params) = state
                    .get_machine(machine_id)
                    .ok_or(Problem::not_found(path))?;
                let compiled_binary = state
                    .get_binary(binary_id)
                    .ok_or(Problem::not_found(path))?;
                (binary_id, params, compiled_binary)
            };

            match method {
                &Method::GET => Ok(build_response(&GetMachine {
                    binary_id,
                    params: params.as_ref(),
                })),
                &Method::POST => {
                    let activation_input = slurp_request_body(&mut req).await?;
                    let output = state
                        .executor()
                        .activate_machine(
                            state.clone(),
                            compiled_binary.as_ref(),
                            params.as_ref(),
                            &activation_input,
                        )
                        .await
                        .map_err(|e| {
                            Problem::new(
                                format!("error occurred while trying to activate machine: {}", e),
                                e,
                                StatusCode::INTERNAL_SERVER_ERROR,
                            )
                        })?
                        .map_err(|e| {
                            Problem::new(
                                format!("machine failed to complete activation: {}", e),
                                e.into(),
                                StatusCode::BAD_REQUEST,
                            )
                        })?;
                    Ok(Response::new(Body::from(output)))
                }
                method => Err(Problem::method_not_allowed(
                    path,
                    method.as_str(),
                    &["POST", "GET"],
                )),
            }
        }
        (method, ["machines"]) => Err(Problem::method_not_allowed(path, method.as_str(), &[])),
        (_, ["machines", machine_id, inner_path @ ..]) => {
            let machine_id = MachineId::from_str(machine_id)
                .map_err(|e| Problem::invalid_path_element::<MachineId>(e.into(), machine_id))?;
            let (params, compiled_binary) = {
                let (binary_id, params) = state
                    .get_machine(machine_id)
                    .ok_or(Problem::not_found(path))?;
                let compiled_binary = state
                    .get_binary(binary_id)
                    .ok_or(Problem::not_found(path))?;
                (params, compiled_binary)
            };

            let transformed_uri = {
                let mut parts = req.uri().clone().into_parts();
                let mut new_paq = format!("/{}", inner_path.join("/"));
                if let Some(paq) = parts.path_and_query {
                    if let Some(query) = paq.query() {
                        new_paq.extend(["?", query]);
                    }
                }
                let new_paq = PathAndQuery::from_str(&new_paq)
                    .with_context(|| format!("trying to turn {new_paq} into a path and query"))
                    .map_err(Problem::internal_server_error)?;
                parts.path_and_query = Some(new_paq);
                Uri::from_parts(parts)
                    .context("trying to transform request URI for machine to handle")
                    .map_err(Problem::internal_server_error)?
            };
            *req.uri_mut() = transformed_uri;
            let output = state
                .executor()
                .machine_handle_http_request(
                    state.clone(),
                    compiled_binary.as_ref(),
                    params.as_ref(),
                    req,
                )
                .await
                .map_err(Problem::internal_server_error)?
                .map_err(Problem::guest_error)?;

            Ok(output)
        }
        _ => Err(Problem::not_found(path)),
    }
}

async fn slurp_request_body(req: &mut Request<Body>) -> Result<Vec<u8>, Problem> {
    let body_stream = req.body_mut();
    let mut buf = Vec::with_capacity(body_stream.size_hint().upper().unwrap_or(0) as usize);

    while let Some(body) = body_stream.data().await {
        match body {
            Ok(body) => buf.extend_from_slice(body.as_ref()),
            Err(e) => {
                return Err(Problem::new(
                    format!("Unable to fetch next chunk of post body: {}", e),
                    e.into(),
                    StatusCode::BAD_REQUEST,
                ))
            }
        }
    }

    Ok(buf)
}

#[derive(Clone)]
pub struct Handler {
    state: State,
    resource_map: Arc<HashMap<String, Vec<u8>>>,
}

impl Handler {
    async fn handle(self, req: Request<Body>) -> Result<Response<Body>, Infallible> {
        if let Some(resource_path) = req.uri().path().strip_prefix("/resources/") {
            return self.handle_resource_request(resource_path).await;
        }
        self._handle(req).await
    }
    async fn handle_resource_request(self, path: &str) -> Result<Response<Body>, Infallible> {
        let span = span!(Level::DEBUG, "resource handler", resource = path);
        let _enter = span.enter();
        match self.resource_map.get(path) {
            Some(resource) => {
                event!(Level::DEBUG, "resource found");
                let mut res = Response::builder().status(200);
                if path.ends_with(".css") {
                    res = res.header("Content-Type", "text/css");
                }
                Ok(res.body(Body::from(resource.clone())).expect("infallible"))
            }
            None => {
                event!(Level::INFO, "resource not found");
                Ok(Response::builder()
                    .status(404)
                    .body(Body::empty())
                    .expect("infallible"))
            }
        }
    }
    async fn _handle(self, req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let span = span!(
            Level::INFO,
            "HTTP",
            method = req.method().as_str(),
            uri = req.uri().to_string()
        );
        match dispatch(req, self.state).instrument(span.clone()).await {
            Ok(res) => Ok(res),
            Err(problem) => {
                let _enter = span.enter();
                event!(
                    Level::DEBUG,
                    error = problem.host_error.to_string(),
                    "HTTP response failed"
                );
                let status = problem.status;
                let body = problem.into_json_body();
                let mut response = Response::new(Body::from(body));
                *response.status_mut() = status;
                Ok(response)
            }
        }
    }
}

pub async fn start(config: config::HttpServerConfig, state: State) -> anyhow::Result<()> {
    let mut resource_map: HashMap<String, Vec<u8>> = config
        .resources
        .into_iter()
        .map(|(uri_path, file_path)| {
            Ok((
                uri_path,
                std::fs::read(&file_path)
                    .context(format!("trying to read {}", file_path.display()))?,
            ))
        })
        .collect::<anyhow::Result<_>>()?;

    resource_map
        .entry("guest-default.css".into())
        .or_insert_with(|| {
            event!(Level::INFO, "using guest-default.css from carol build");
            include_bytes!("../../resources/guest-default.css").to_vec()
        });
    let resource_map = Arc::new(resource_map);
    let handler = Handler {
        state,
        resource_map,
    };

    // And a MakeService to handle each connection...
    let make_service = make_service_fn(move |_conn| {
        let handler = handler.clone();
        let service = move |req| handler.clone().handle(req);
        async move { Ok::<_, Infallible>(service_fn(service)) }
    });

    event!(Level::INFO, "Binding http server to {}", config.listen);

    // Then bind and serve...
    Ok(Server::bind(&config.listen).serve(make_service).await?)
}
