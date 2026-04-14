# Vector Search Proof
This project compares the performance of two vector search algorithms: float-based cosine similarity search and constraint theory-based search using PythagoreanManifold.

## How to run
1. Install Rust and cargo.
2. Run `cargo run --release` in the project directory.
3. The program will generate 10000 random 128-dim vectors and run 200 queries.
4. The results will be printed to the console, including recall@10, bytes per vector, and query latency.
