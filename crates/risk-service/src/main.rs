//! Risk Service Entry Point

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Create service (stateless - no DB required)
    let risk_svc = risk_service::RiskServiceImpl::default();

    // Start gRPC server
    let port = std::env::var("PORT").unwrap_or_else(|_| "50005".to_string());
    let addr: std::net::SocketAddr = format!("[::1]:{}", port).parse()?;
    tracing::info!("Risk service listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(api::risk::risk_service_server::RiskServiceServer::new(risk_svc))
        .serve(addr)
        .await?;

    Ok(())
}
