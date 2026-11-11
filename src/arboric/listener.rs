//! The main proxy listener: binds a TCP socket and serves each connection
//! with a [ProxyService](super::ProxyService)
//!
use super::ProxyService;
use super::proxy_service::HttpClient;
use crate::config::ListenerConfig;
use http::Uri;
use hyper::service::service_fn;
use hyper_util::client::legacy::Client;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use hyper_util::server::graceful::GracefulShutdown;
use log::{debug, info, trace, warn};
use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;

/// How long to wait for open connections to finish after shutdown starts
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);

/// The main Proxy/Listener
#[derive(Debug)]
pub struct Listener {
    context: Arc<ListenerContext>,
}

#[derive(Debug)]
pub struct ListenerContext {
    pub listener_address: SocketAddr,
    pub listener_path: Option<String>,
    pub api_uri: Uri,
    pub pdp: crate::abac::PDP,
    pub influx_db_backend: Option<super::influxdb::Backend>,
    pub secret_key_bytes: Option<Vec<u8>>,
    pub client: HttpClient,
}

impl Listener {
    /// Constructs a new Listener with the given backend API URI
    pub fn new(listener_config: ListenerConfig) -> Self {
        let secret_key_bytes;
        if let Some(key_source) = &listener_config.jwt_signing_key_source {
            match key_source.get_secret_key_bytes() {
                Ok(bytes) => {
                    trace!("secret_key_bytes => {:?}", bytes);
                    secret_key_bytes = Some(bytes);
                }
                Err(err) => panic!("Enable to get secret key bytes: {}!", err),
            }
        } else {
            secret_key_bytes = None;
        }
        let client = Client::builder(TokioExecutor::new()).build_http();
        let context = ListenerContext {
            listener_address: listener_config.listener_address,
            listener_path: listener_config.listener_path,
            api_uri: listener_config.api_uri,
            pdp: listener_config.pdp,
            influx_db_backend: listener_config.influx_db_backend,
            secret_key_bytes,
            client,
        };
        Listener {
            context: Arc::new(context),
        }
    }

    /// Binds the configured address. Use port 0 to get a free port, then
    /// read it with [BoundListener::local_addr].
    pub async fn bind(self) -> io::Result<BoundListener> {
        let listener = TcpListener::bind(&self.context.listener_address).await?;
        info!("Proxy listening on {}", listener.local_addr()?);
        Ok(BoundListener {
            listener,
            context: self.context,
        })
    }

    /// Binds and serves until Ctrl-C
    pub async fn run(self) -> io::Result<()> {
        let bound = self.bind().await?;
        bound
            .serve(async {
                if let Err(err) = tokio::signal::ctrl_c().await {
                    warn!("Unable to listen for Ctrl-C: {}", err);
                    std::future::pending::<()>().await;
                }
            })
            .await
    }
}

/// A [Listener] that has bound its TCP socket
#[derive(Debug)]
pub struct BoundListener {
    listener: TcpListener,
    context: Arc<ListenerContext>,
}

impl BoundListener {
    /// The address the listener is bound to
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    /// Accepts and serves connections until `shutdown` completes. Then stops
    /// accepting, and waits for open connections to finish (up to
    /// [SHUTDOWN_TIMEOUT]).
    pub async fn serve(self, shutdown: impl Future<Output = ()>) -> io::Result<()> {
        let builder = auto::Builder::new(TokioExecutor::new());
        let graceful = GracefulShutdown::new();
        let service = ProxyService::new(self.context.clone());
        tokio::pin!(shutdown);

        loop {
            tokio::select! {
                accepted = self.listener.accept() => {
                    let (stream, peer) = match accepted {
                        Ok(accepted) => accepted,
                        Err(err) => {
                            warn!("Unable to accept connection: {}", err);
                            continue;
                        }
                    };
                    debug!("Accepted connection from {}", peer);
                    let service = service.clone();
                    let conn = builder.serve_connection(
                        TokioIo::new(stream),
                        service_fn(move |req| {
                            let service = service.clone();
                            async move { service.call(req).await }
                        }),
                    );
                    let conn = graceful.watch(conn.into_owned());
                    tokio::spawn(async move {
                        if let Err(err) = conn.await {
                            debug!("Connection from {} closed with error: {}", peer, err);
                        }
                    });
                }
                _ = &mut shutdown => {
                    info!("Shutting down");
                    break;
                }
            }
        }

        drop(self.listener);
        tokio::select! {
            _ = graceful.shutdown() => debug!("All connections closed"),
            _ = tokio::time::sleep(SHUTDOWN_TIMEOUT) => {
                warn!("Timed out waiting for connections to close");
            }
        }
        Ok(())
    }
}
