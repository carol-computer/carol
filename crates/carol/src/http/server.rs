use super::api::{self, *};
use super::resolver::{Resolution, Resolver};
use crate::config;
use anyhow::{anyhow, Context};
use carol_core::{hex, BinaryId, MachineId};
use carol_host::{CompiledBinary, GuestError, State};
use hyper::http::uri::PathAndQuery;
use hyper::http::HeaderValue;
use hyper::service::{make_service_fn, service_fn};
use hyper::{body::HttpBody, Body, Method, Request, Response, Server, StatusCode};
use hyper::{header, Uri};
use std::collections::HashMap;
use std::convert::Infallible;
use std::future::Future;
use std::net::SocketAddr;
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

    pub fn bad_request(client_desc: impl Into<String>, host_error: anyhow::Error) -> Self {
        Self::new(client_desc.into(), host_error, StatusCode::BAD_REQUEST)
    }

    pub fn internal_server_error(host_error: anyhow::Error) -> Self {
        Self::new(
            "internal server error".into(),
            host_error,
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    }

    pub fn misdirected_request(host: &HeaderValue) -> Self {
        let host = host
            .to_str()
            .map(|x| x.to_string())
            .unwrap_or_else(|_| format!("hex:\"{}\"", hex::encode(host.as_bytes())));
        Self::new(
            format!("HOST {host} couldn't be resovled to a machine"),
            anyhow!("HOST {host} couldn't be resovled to a machine"),
            StatusCode::MISDIRECTED_REQUEST,
        )
    }

    pub fn not_found(path: &str) -> Self {
        Self::new(
            format!("{} not found", path),
            anyhow!("resource not found: {}", path),
            StatusCode::NOT_FOUND,
        )
    }

    pub fn machine_not_found(machine_id: MachineId) -> Self {
        Self::new(
            format!("machine {machine_id} not found"),
            anyhow!("machine {machine_id} not found"),
            StatusCode::NOT_FOUND,
        )
    }

    pub fn binary_not_found(binary_id: BinaryId) -> Self {
        Self::new(
            format!("binary {binary_id} not found"),
            anyhow!("binary {binary_id} not found"),
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
    resolver: Resolver,
}

impl Handler {
    async fn handle(self, req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let host = req
            .headers()
            .get(header::HOST)
            .and_then(|host| host.to_str().ok())
            .unwrap_or("<unable-to-decode>");
        let span = span!(
            Level::INFO,
            "HTTP",
            method = req.method().as_str(),
            uri = req.uri().to_string(),
            host = host,
        );

        match self.dispatch(req).instrument(span.clone()).await {
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
                response
                    .headers_mut()
                    .append(header::CONTENT_TYPE, "application/json".parse().unwrap());
                *response.status_mut() = status;
                Ok(response)
            }
        }
    }

    fn machine_components(
        &self,
        id: MachineId,
    ) -> Result<(BinaryId, Arc<Vec<u8>>, Arc<CompiledBinary>), Problem> {
        let (binary_id, params) = self
            .state
            .exec
            .get_machine(id)
            .ok_or(Problem::machine_not_found(id))?;
        let compiled_binary = self
            .state
            .exec
            .get_binary(binary_id)
            .ok_or(Problem::binary_not_found(binary_id))?;
        Ok((binary_id, params, compiled_binary))
    }

    async fn http_request_to_machine(
        &self,
        id: MachineId,
        request: Request<Body>,
    ) -> Result<Response<Body>, Problem> {
        let (_, params, compiled_binary) = self.machine_components(id)?;
        let output = self
            .state
            .exec
            .executor()
            .machine_handle_http_request(
                self.state.clone(),
                compiled_binary.as_ref(),
                params.as_ref(),
                request,
            )
            .await
            .map_err(Problem::internal_server_error)?
            .map_err(Problem::guest_error)?;

        Ok(output)
    }

    pub async fn dispatch(&self, mut req: Request<Body>) -> Result<Response<Body>, Problem> {
        let state = &self.state;

        match req.headers().get(header::HOST) {
            Some(host_header) => {
                let resolution = self
                    .resolver
                    .resolve_host(host_header)
                    .await
                    .map_err(|e| Problem::internal_server_error(e))?;
                match resolution {
                    Resolution::Api => { /* carry on */ }
                    Resolution::Unknown => return Err(Problem::misdirected_request(host_header)),
                    Resolution::Machine(machine_id) => {
                        return self.http_request_to_machine(machine_id, req).await
                    }
                }
            }
            None => { /* assume it's for API */ }
        }

        let path = req.uri().path();

        let segments = {
            let mut segments = path.split('/');
            // ignore first `/`
            let _ = segments.next();
            segments.collect::<Vec<_>>()
        };

        match (req.method(), &segments[..]) {
            (&Method::GET, [""]) => Ok(build_response(&Root {
                static_public_key: state.bls_keypair.public_key(),
                base_domain: self.resolver.base_domain().map(ToString::to_string),
            })),
            (&Method::POST, ["binaries"]) => {
                let body = slurp_request_body(&mut req).await?;
                let binary_id = BinaryId::new(&body);
                let already_exists = state.exec.get_binary(binary_id).is_some();
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
                        .exec
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
                    state.exec.insert_binary(compiled_binary);
                    event!(Level::INFO, "new binary uploaded");
                    Ok(build_response(&BinaryCreated { id: binary_id }))
                }
            }
            (method, ["binaries", binary_id]) => {
                let binary_id = BinaryId::from_str(binary_id)
                    .map_err(|e| Problem::invalid_path_element::<BinaryId>(e.into(), binary_id))?;
                let binary = state
                    .exec
                    .get_binary(binary_id)
                    .ok_or(Problem::binary_not_found(binary_id))?;

                match method {
                    &Method::GET => {
                        let carol_host::guest::BinaryApi { activations } = state
                            .exec
                            .executor()
                            .get_binary_api(&binary)
                            .await
                            .map_err(|e| {
                                Problem::bad_request("failed to retrieve API from binary", e)
                            })?;
                        let response = build_response(&carol_http::api::BinaryDescription {
                            activations: activations
                                .into_iter()
                                .map(|carol_host::guest::ActivationDescription { name }| {
                                    (name, carol_http::api::AcivationDescription {})
                                })
                                .collect(),
                        });
                        Ok(response)
                    }
                    &Method::POST => {
                        let params = slurp_request_body(&mut req).await?;
                        let (already_exists, machine_id) =
                            state.exec.insert_machine(binary_id, params);
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
            (method, ["machines", machine_id, trailing @ ..]) => {
                let machine_id = MachineId::from_str(machine_id).map_err(|e| {
                    Problem::invalid_path_element::<MachineId>(e.into(), machine_id)
                })?;

                match trailing {
                    &[] => match method {
                        &Method::GET => {
                            let (binary_id, params, _) = self.machine_components(machine_id)?;
                            Ok(build_response(&GetMachine {
                                binary_id,
                                params: params.as_ref(),
                            }))
                        }
                        _ => Err(Problem::method_not_allowed(path, method.as_str(), &["GET"])),
                    },
                    ["http", inner_path @ ..] => {
                        // we need to direct /http to /http/ so relative urls work in the machine
                        if inner_path.is_empty() && !req.uri().path().ends_with('/') {
                            return Ok(Response::builder()
                                .header(header::LOCATION, "http/")
                                .status(StatusCode::PERMANENT_REDIRECT)
                                .body(Body::empty())
                                .unwrap());
                        }
                        let transformed_uri = {
                            let mut parts = req.uri().clone().into_parts();
                            let mut new_paq = format!("/{}", inner_path.join("/"));
                            if let Some(paq) = parts.path_and_query {
                                if let Some(query) = paq.query() {
                                    new_paq.extend(["?", query]);
                                }
                            }
                            let new_paq = PathAndQuery::from_str(&new_paq)
                                .with_context(|| {
                                    format!("trying to turn {new_paq} into a path and query")
                                })
                                .map_err(Problem::internal_server_error)?;
                            parts.path_and_query = Some(new_paq);
                            Uri::from_parts(parts)
                                .context("trying to transform request URI for machine to handle")
                                .map_err(Problem::internal_server_error)?
                        };
                        *req.uri_mut() = transformed_uri;

                        self.http_request_to_machine(machine_id, req).await
                    }
                    ["activate", activation_name] => {
                        let (_, params, compiled_binary) = self.machine_components(machine_id)?;
                        let activation_name = activation_name.to_string();
                        match method {
                            &Method::POST => {
                                let activation_input = slurp_request_body(&mut req).await?;
                                let output = state
                                    .exec
                                    .executor()
                                    .activate_machine(
                                        state.clone(),
                                        compiled_binary.as_ref(),
                                        params.as_ref(),
                                        &activation_name,
                                        &activation_input,
                                    )
                                    .await
                                    .map_err(|e| {
                                        Problem::new(
                                            format!(
                                                "error occurred while trying to activate machine: {}",
                                                e
                                            ),
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
                                &["POST"],
                            )),
                        }
                    }
                    _ => Err(Problem::not_found(path)),
                }
            }
            (method, ["machines"]) => Err(Problem::method_not_allowed(path, method.as_str(), &[])),
            _ => Err(Problem::not_found(path)),
        }
    }
}

pub fn start(
    config: config::HttpServerConfig,
    state: State,
) -> anyhow::Result<(SocketAddr, impl Future<Output = ()> + Send + Sync + 'static)> {
    let handler = Handler {
        state,
        resolver: config.dns.into_resolver(),
    };

    // And a MakeService to handle each connection...
    let make_service = make_service_fn(move |_conn| {
        let handler = handler.clone();
        let service = move |req| handler.clone().handle(req);
        async move { Ok::<_, Infallible>(service_fn(service)) }
    });

    event!(Level::DEBUG, "Try to bind http server to {}", config.listen);

    // bind first so we can figure out which port we actually listened on
    let server = Server::bind(&config.listen).serve(make_service);
    let local_addr = server.local_addr();
    let server = async {
        match server.await {
            Ok(_) => event!(Level::INFO, "HTTP server shut down"),
            Err(e) => event!(
                Level::ERROR,
                error = e.to_string(),
                "HTTP server unexpectedly shut down"
            ),
        }
    };
    Ok((local_addr, server))
}
