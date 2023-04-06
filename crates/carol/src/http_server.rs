use crate::config;
use carol_core::FullActivation;
use carol_host::Executor;
use hyper::service::{make_service_fn, service_fn};
use hyper::{body::HttpBody, Body, Method, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use tracing::{event, Level};

async fn handle(mut req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/activate") => {
            let executor = Executor::new();
            let body_stream = req.body_mut();
            let mut buf =
                Vec::with_capacity(body_stream.size_hint().upper().unwrap_or(1024) as usize);

            while let Some(body) = body_stream.data().await {
                match body {
                    Ok(body) => buf.extend_from_slice(body.as_ref()),
                    Err(_) => {
                        let mut response = Response::new(Body::empty());
                        *response.status_mut() = StatusCode::BAD_REQUEST;
                        return Ok(response);
                    }
                }
            }

            match bincode::borrow_decode_from_slice::<FullActivation, _>(
                &buf,
                bincode::config::standard(),
            ) {
                Ok((binary_post, _)) => {
                    match executor.load_contract_from_binary(binary_post.binary) {
                        Ok(contract) => {
                            match executor
                                .execute_contract(
                                    contract,
                                    binary_post.parameters,
                                    binary_post.activation_input,
                                )
                                .await
                            {
                                Ok(output) => Ok(Response::new(Body::from(output))),
                                Err(e) => {
                                    event!(Level::ERROR, "Running WASM binary failed: {}", e);
                                    let mut response = Response::new(Body::empty());
                                    *response.status_mut() = StatusCode::BAD_REQUEST;
                                    return Ok(response);
                                }
                            }
                        }
                        Err(e) => {
                            event!(Level::ERROR, "Invalid WASM binary: {}", e);
                            let mut response = Response::new(Body::empty());
                            *response.status_mut() = StatusCode::BAD_REQUEST;
                            return Ok(response);
                        }
                    }
                }
                Err(e) => {
                    event!(Level::ERROR, "Unable to decode binary post: {}", e);
                    let mut response = Response::new(Body::empty());
                    *response.status_mut() = StatusCode::BAD_REQUEST;
                    return Ok(response);
                }
            }
        }
        _ => {
            let mut response = Response::new(Body::empty());
            *response.status_mut() = StatusCode::NOT_FOUND;
            Ok(response)
        }
    }
}

pub async fn start(config: config::HttpServerConfig) -> Result<(), hyper::Error> {
    // And a MakeService to handle each connection...
    let make_service = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });

    event!(Level::INFO, "Binding http server to {}", config.listen);

    // Then bind and serve...
    Server::bind(&config.listen).serve(make_service).await
}
