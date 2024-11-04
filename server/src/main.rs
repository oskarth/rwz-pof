mod handlers;
mod storage;

use handlers::{handle_commitment, handle_proof, handle_verify};
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use storage::Storage;
use warp::Filter;

#[tokio::main]
async fn main() {
    // Initialize basic logging
    tracing_subscriber::fmt::init();
    println!("Starting RWZ-POF server...");

    // Initialize storage with thread-safe wrapper
    let storage = Arc::new(Mutex::new(Storage::new()));
    let storage_filter = warp::any().map(move || storage.clone());

    // POST /lb/commitment
    let commitment = warp::post()
        .and(warp::path("lb"))
        .and(warp::path("commitment"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(storage_filter.clone())
        .and_then(handle_commitment);

    // POST /bb/proof
    let proof = warp::post()
        .and(warp::path("bb"))
        .and(warp::path("proof"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(storage_filter.clone())
        .and_then(handle_proof);

    // POST /sb/verify
    let verify = warp::post()
        .and(warp::path("sb"))
        .and(warp::path("verify"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(storage_filter)
        .and_then(handle_verify);

    // Combine routes
    let routes = commitment.or(proof).or(verify);

    // Start the server
    println!("Server running on http://localhost:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
