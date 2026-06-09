//! Streamable HTTP transport — the "public" hosting path.
//!
//! Builds an axum app that mounts `rmcp`'s Streamable HTTP service at `/mcp`,
//! fronted by a bearer-token gate. A fresh [`BwocMcp`] handler is produced per
//! session from the shared [`Bridge`] + [`Posture`].
//!
//! NOTE: the exact `rmcp::transport::streamable_http_server` constructor surface
//! shifts across `rmcp` minor versions — confirm against the pinned 1.7 docs
//! when first building with `--features http` (see `docs/PLAN.md` §Phase 2).

use crate::bridge::Bridge;
use crate::cli::{Cli, Posture};
use crate::server::BwocMcp;
use axum::http::{HeaderMap, StatusCode};
use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};
use std::sync::Arc;

pub async fn serve(args: Cli, bridge: Bridge, posture: Posture) -> anyhow::Result<()> {
    // Refuse to expose a mutating surface without authentication.
    let mutating = posture.write || posture.exec || posture.dangerous;
    if args.http_token.is_none() && !args.http_insecure {
        anyhow::bail!(
            "HTTP transport needs --http-token (or explicit --http-insecure for read-only loopback)"
        );
    }
    if args.http_token.is_none() && mutating {
        anyhow::bail!(
            "refusing to serve write/exec tools over HTTP without --http-token"
        );
    }

    let token = args.http_token.clone();
    let mcp = StreamableHttpService::new(
        move || Ok(BwocMcp::new(bridge.clone(), posture)),
        Arc::new(LocalSessionManager::default()),
        Default::default(),
    );

    let app = axum::Router::new()
        .nest_service("/mcp", mcp)
        .layer(axum::middleware::from_fn(move |headers: HeaderMap, req, next| {
            let token = token.clone();
            async move { auth_gate(token, headers, req, next).await }
        }));

    let listener = tokio::net::TcpListener::bind(&args.http_addr).await?;
    tracing::info!(addr = %args.http_addr, "bwoc-mcp HTTP listening on /mcp");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn auth_gate(
    expected: Option<String>,
    headers: HeaderMap,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, StatusCode> {
    if let Some(expected) = expected {
        let ok = headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(|t| t == expected)
            .unwrap_or(false);
        if !ok {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }
    Ok(next.run(req).await)
}
