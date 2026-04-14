use rand::Rng;
use std::time::Instant;
use constraint_theory_core::PythagoreanManifold;

fn main() {
    let modes = vec!["float", "ct"];
    let results = modes.into_iter().map(|mode| {
        let mut vectors = vec![];
        let mut rng = rand::thread_rng();
        for _ in 0..10000 {
            let mut vector = [0.0f64; 128];
            for i in 0..128 {
                vector[i] = rng.gen::<f64>();
            }
            vectors.push(vector);
        }
        let queries = vectors.clone();

        let mut recall = 0.0;
        let mut query_latency = 0.0;
        let mut bytes_per_vector = 0;
        match mode {
            "float" => {
                bytes_per_vector = 128 * 8;
                let start = Instant::now();
                for query in queries.iter().take(200) {
                    let mut top_k = vec![];
                    for vector in vectors.iter() {
                        let mut dot_product = 0.0;
                        for i in 0..128 {
                            dot_product += query[i] * vector[i];
                        }
                        let magnitude_query = (query.iter().map(|x| x*x).sum::<f64>()).sqrt();
                        let magnitude_vector = (vector.iter().map(|x| x*x).sum::<f64>()).sqrt();
                        let similarity = dot_product / (magnitude_query * magnitude_vector);
                        top_k.push((similarity, vector));
                    }
                    top_k.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
                    recall += top_k.iter().take(10).filter(|(sim, vec)| **vec == queries[0]).count() as f64;
                }
                query_latency = start.elapsed().as_millis() as f64 / 200.0;
            },
            "ct" => {
                let manifold = PythagoreanManifold::new(10);
                bytes_per_vector = 128 * 4;
                let start = Instant::now();
                for query in queries.iter().take(200) {
                    let query_snapped: Vec<[f32; 2]> = query.chunks(2).map(|chunk| {
                        let pair: [f32; 2] = [chunk[0] as f32, chunk[1] as f32];
                        manifold.snap(pair).0
                    }).collect();
                    let mut top_k = vec![];
                    for vector in vectors.iter() {
                        let vector_snapped: Vec<[f32; 2]> = vector.chunks(2).map(|chunk| {
                            let pair: [f32; 2] = [chunk[0] as f32, chunk[1] as f32];
                            manifold.snap(pair).0
                        }).collect();
                        let mut dot_product = 0.0;
                        for i in 0..64 {
                            dot_product += (query_snapped[i][0] * vector_snapped[i][0]) + (query_snapped[i][1] * vector_snapped[i][1]);
                        }
                        let magnitude_query = (query_snapped.iter().map(|x| x[0]*x[0] + x[1]*x[1]).sum::<f32>()).sqrt();
                        let magnitude_vector = (vector_snapped.iter().map(|x| x[0]*x[0] + x[1]*x[1]).sum::<f32>()).sqrt();
                        let similarity = dot_product / (magnitude_query * magnitude_vector);
                        top_k.push((similarity, vector));
                    }
                    top_k.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
                    recall += top_k.iter().take(10).filter(|(sim, vec)| **vec == queries[0]).count() as f64;
                }
                query_latency = start.elapsed().as_millis() as f64 / 200.0;
            },
            _ => {},
        }
        (mode, recall / 200.0, bytes_per_vector, query_latency)
    }).collect::<Vec<_>>();
    println!("Mode\tRecall@10\tBytes per vector\tQuery latency");
    for (mode, recall, bytes_per_vector, query_latency) in results {
        println!("{}\t{:?}\t{:?}\t{:?}", mode, recall, bytes_per_vector, query_latency);
    }
}