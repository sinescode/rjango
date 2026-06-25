//! Start the development server.
//! Mirrors `rjango runserver`.

use std::sync::Arc;

pub fn run(addr: &std::net::SocketAddr, app: rjango_server::Application) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    rt.block_on(async {
        tracing::info!("Rjango development server starting at http://{}", addr);
        rjango_server::run_server(Arc::new(app), *addr).await.unwrap();
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_runserver_fn_exists() {
        let _ = super::run;
    }
}
