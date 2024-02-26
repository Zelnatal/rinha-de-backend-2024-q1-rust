use config::create_app;

mod config;
mod models;
mod routes;

#[tokio::main]
async fn main() {
    let app = create_app().await;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener,app).await.unwrap();
}
