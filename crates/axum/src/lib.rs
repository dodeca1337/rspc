//! rspc-axum: Axum integration for [rspc](https://rspc.dev).
#![cfg_attr(docsrs2, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png",
    html_favicon_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png"
)]

use std::{collections::HashMap, sync::Arc};

use axum::{
    body::{to_bytes, Body},
    extract::Request,
    http::{request::Parts, Method, Response, StatusCode},
    response::IntoResponse,
    routing::{on, MethodFilter},
    RequestExt, Router,
};
use drspc::internal::{
    jsonrpc::{self, handle_json_rpc, RequestId, Sender, SubscriptionMap},
    ProcedureKind,
};
use extractors::TCtxFunc;

mod extractors;

pub fn endpoint<TCtx, TCtxFnMarker, TCtxFn, S>(
    router: Arc<drspc::Router<TCtx>>,
    ctx_fn: TCtxFn,
) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    TCtx: Send + Sync + 'static,
    TCtxFnMarker: Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
{
    Router::<S>::new().route(
        "/:id",
        on(
            MethodFilter::GET.or(MethodFilter::POST),
            move |req: axum::extract::Request<Body>| {
                let router = router.clone();

                async move {
                    let ctx_fn = ctx_fn.clone();

                    match (req.method(), &req.uri().path()[1..]) {
                        (&Method::GET, "ws") => {
                            {
                                let mut req = req;
                                return req
                                    .extract_parts::<axum::extract::ws::WebSocketUpgrade>()
                                    .await
                                    .unwrap() // TODO: error handling
                                    .on_upgrade(|socket| {
                                        handle_websocket(ctx_fn, socket, req.into_parts().0, router)
                                    })
                                    .into_response();
                            }
                        }
                        _ => {
                            Response::builder()
                                .status(StatusCode::NOT_FOUND)
                                .body(Body::from("[]")) // TODO: Better error message which frontend is actually setup to handle.
                                .unwrap()
                        }
                    }
                }
            },
        ),
    )
}

async fn handle_websocket<TCtx, TCtxFn, TCtxFnMarker>(
    ctx_fn: TCtxFn,
    mut socket: axum::extract::ws::WebSocket,
    parts: Parts,
    router: Arc<drspc::Router<TCtx>>,
) where
    TCtx: Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
{
    use axum::extract::ws::Message;
    use futures::StreamExt;
    use tokio::sync::mpsc;

    #[cfg(feature = "tracing")]
    tracing::debug!("Accepting websocket connection");

    let mut subscriptions = HashMap::new();
    let (mut tx, mut rx) = mpsc::channel::<jsonrpc::Response>(100);
    loop {
        tokio::select! {
            biased; // Note: Order is important here
            msg = rx.recv() => {
                match socket.send(Message::Binary(match rmp_serde::to_vec_named(&msg) {
                    Ok(v) => v,
                    Err(_err) => {
                        #[cfg(feature = "tracing")]
                        tracing::error!("Error serializing websocket message: {}", _err);

                        continue;
                    }
                })).await {
                    Ok(_) => {}
                    Err(_err) => {
                        #[cfg(feature = "tracing")]
                        tracing::error!("Error sending websocket message: {}", _err);

                        continue;
                    }
                }
            }


            msg = socket.next() => {
                match msg {
                    Some(Ok(msg)) => {
                       let res = match msg {
                            Message::Text(text) => rmp_serde::from_slice::<jsonrpc::Request>(text.as_bytes()),
                            Message::Binary(binary) => rmp_serde::from_slice(&binary),
                            Message::Ping(_) | Message::Pong(_) | Message::Close(_) => {
                                continue;
                            }
                        };

                        match res {
                            Ok(request) => {
                                let ctx = ctx_fn.exec(parts.clone());

                                handle_json_rpc(match ctx {
                                    Ok(v) => v,
                                    Err(_err) => {
                                        #[cfg(feature = "tracing")]
                                        tracing::error!("Error executing context function: {}", _err);

                                        continue;
                                    }
                                }, request, &router, &mut Sender::Channel(&mut tx),
                                &mut SubscriptionMap::Ref(&mut subscriptions)).await;
                            },
                            Err(_err) => {
                                #[cfg(feature = "tracing")]
                                tracing::error!("Error parsing websocket message: {}", _err);

                                // TODO: Send report of error to frontend

                                continue;
                            }
                        };
                    }
                    Some(Err(_err)) => {
                        #[cfg(feature = "tracing")]
                        tracing::error!("Error in websocket: {}", _err);

                        // TODO: Send report of error to frontend

                        continue;
                    },
                    None => {
                        #[cfg(feature = "tracing")]
                        tracing::debug!("Shutting down websocket connection");

                        // TODO: Send report of error to frontend

                        return;
                    },
                }
            }
        }
    }
}
