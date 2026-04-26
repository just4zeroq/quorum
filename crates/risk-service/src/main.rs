//! Risk Service Entry Point

use registry::ServiceRegistry;
use risk_service::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Load config
    let config = Config::default();

    // Create service (stateless - no DB required)
    let risk_svc = risk_service::RiskServiceImpl::default();

    // Start gRPC server
    let addr: std::net::SocketAddr = format!("{}:{}", config.service.host, config.service.port).parse()?;
    tracing::info!("Risk service listening on {}", addr);

    // Register to etcd
    let registry = ServiceRegistry::new(
        "risk-service",
        &format!("http://{}", addr),
        &config.etcd_endpoints,
    )
    .await
    .map_err(|e| format!("Failed to create service registry: {}", e))?;
    registry
        .register(30)
        .await
        .map_err(|e| format!("Failed to register service: {}", e))?;
    let _heartbeat_handle = registry.clone().start_heartbeat(30, 10);

    tonic::transport::Server::builder()
        .add_service(api::risk::risk_service_server::RiskServiceServer::new(risk_svc))
        .serve(addr)
        .await?;

    Ok(())
}