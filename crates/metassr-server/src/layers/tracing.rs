/// Tracing layer for MetaSSR internal server
use axum::{
    http::{HeaderValue, Request},
    response::Response,
};

use tokio::time::Duration;
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::{debug, error, Span};

use crate::router::RouterMut;

pub trait LayerSetup {
    type LayerOptions;
    fn setup<S: Clone + Send + Sync + 'static>(options: Self::LayerOptions, app: &mut RouterMut<S>);
}

#[derive(Debug)]
pub struct TracingLayerOptions {
    pub enable_http_logging: bool,
}

#[derive(Clone, Copy)]
pub struct TracingLayer;

impl LayerSetup for TracingLayer {
    type LayerOptions = TracingLayerOptions;
    fn setup<S: Clone + Send + Sync + 'static>(
        options: Self::LayerOptions,
        app: &mut RouterMut<S>,
    ) {
        let trace_layer = TraceLayer::new_for_http().on_failure(
            |err: ServerErrorsFailureClass, latency: Duration, _span: &Span| {
                error!(
                    target = "http",
                    latency = format!("{latency:?}"),
                    "Server failure: {err}",
                )
            },
        )
        .on_request(move |req: &Request<_>, _span: &Span| {
            if options.enable_http_logging {
                debug!(
                    target = "http",
                    user_agent=?  req.headers().get("user-agent").unwrap_or(&HeaderValue::from_str("Unknown").unwrap()),
                    "request: {} {}",
                    req.method(),
                    req.uri().path(),
                )
            }
        })
        .on_response(move |res: &Response<_>, latency: Duration, _span: &Span| {
            if options.enable_http_logging {
                debug!(
                    target = "http",
                    "[{}]: generated in {:?}",
                    res.status(),
                    latency,
                )
            }
        });

        app.layer(trace_layer);
    }
}
