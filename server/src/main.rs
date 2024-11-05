mod handlers;
mod storage;
pub mod worker;

use handlers::{
    handle_commitment, handle_create_proof_job, handle_get_proof_job, handle_proof, handle_verify,
};
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use storage::Storage;
use warp::Filter;
use worker::ProofWorker;

// Helper functions for warp filters
fn with_storage(
    storage: Arc<Mutex<Storage>>,
) -> impl Filter<Extract = (Arc<Mutex<Storage>>,), Error = Infallible> + Clone {
    warp::any().map(move || storage.clone())
}

fn with_proof_worker(
    worker: Arc<ProofWorker>,
) -> impl Filter<Extract = (Arc<ProofWorker>,), Error = Infallible> + Clone {
    warp::any().map(move || worker.clone())
}

#[tokio::main]
async fn main() {
    // Initialize basic logging
    tracing_subscriber::fmt::init();
    println!("Starting RWZ-POF server...");

    // Initialize storage with thread-safe wrapper
    let storage = Arc::new(Mutex::new(Storage::new()));

    // Initialize proof worker
    let proof_worker = Arc::new(ProofWorker::new());

    // POST /lb/commitment
    let commitment = warp::post()
        .and(warp::path("lb"))
        .and(warp::path("commitment"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_storage(storage.clone()))
        .and_then(handle_commitment);

    // POST /bb/proof
    let proof = warp::post()
        .and(warp::path("bb"))
        .and(warp::path("proof"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_storage(storage.clone()))
        .and_then(handle_proof);

    // POST /sb/verify
    let verify = warp::post()
        .and(warp::path("sb"))
        .and(warp::path("verify"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_storage(storage.clone()))
        .and_then(handle_verify);

    let create_proof_job = warp::post()
        .and(warp::path("proofs"))
        .and(warp::path("async"))
        .and(warp::body::json())
        .and(with_storage(storage.clone()))
        .and(with_proof_worker(proof_worker.clone()))
        .and_then(handle_create_proof_job);

    let get_proof_job = warp::get()
        .and(warp::path("proofs"))
        .and(warp::path("async"))
        .and(warp::path::param())
        .and(with_storage(storage.clone()))
        .and_then(handle_get_proof_job);

    // Combine routes
    let routes = commitment
        .or(proof)
        .or(verify)
        .or(create_proof_job)
        .or(get_proof_job);

    // Start the server
    println!("Server running on http://localhost:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
