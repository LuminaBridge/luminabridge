//! LuminaBridge Benchmark Suite
//! 
//! Performance benchmarks for core LuminaBridge functionality.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use luminabridge::config::Config;

/// Benchmark JWT token encoding
fn bench_jwt_encoding(c: &mut Criterion) {
    let config = Config::development();
    let claims = serde_json::json!({
        "sub": "user-123",
        "email": "test@example.com",
        "exp": chrono::Utc::now().timestamp() + 3600,
    });

    c.bench_function("jwt_encoding", |b| {
        b.iter(|| {
            let _ = jsonwebtoken::encode(
                &jsonwebtoken::Header::default(),
                &claims,
                &jsonwebtoken::EncodingKey::from_secret(config.oauth.jwt_secret.as_bytes()),
            );
        });
    });
}

/// Benchmark JWT token decoding
fn bench_jwt_decoding(c: &mut Criterion) {
    let config = Config::development();
    let claims = serde_json::json!({
        "sub": "user-123",
        "email": "test@example.com",
        "exp": chrono::Utc::now().timestamp() + 3600,
    });

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(config.oauth.jwt_secret.as_bytes()),
    ).unwrap();

    c.bench_function("jwt_decoding", |b| {
        b.iter(|| {
            let _ = jsonwebtoken::decode::<serde_json::Value>(
                &token,
                &jsonwebtoken::DecodingKey::from_secret(config.oauth.jwt_secret.as_bytes()),
                &jsonwebtoken::Validation::default(),
            );
        });
    });
}

/// Benchmark password hashing
fn bench_password_hashing(c: &mut Criterion) {
    let password = "TestPassword123!";

    c.bench_function("password_hashing", |b| {
        b.iter(|| {
            let _ = argon2::hash_encoded(
                password.as_bytes(),
                &argon2::password_hash::SaltString::generate(&mut rand::thread_rng()),
                &argon2::Params::default(),
            );
        });
    });
}

/// Benchmark password verification
fn bench_password_verification(c: &mut Criterion) {
    let password = "TestPassword123!";
    let hashed = argon2::hash_encoded(
        password.as_bytes(),
        &argon2::password_hash::SaltString::generate(&mut rand::thread_rng()),
        &argon2::Params::default(),
    ).unwrap();

    c.bench_function("password_verification", |b| {
        b.iter(|| {
            let _ = argon2::verify_encoded(&hashed, password.as_bytes());
        });
    });
}

/// Benchmark UUID generation
fn bench_uuid_generation(c: &mut Criterion) {
    c.bench_function("uuid_v4_generation", |b| {
        b.iter(|| {
            let _ = uuid::Uuid::new_v4();
        });
    });
}

/// Benchmark JSON serialization
fn bench_json_serialization(c: &mut Criterion) {
    let data = serde_json::json!({
        "id": "123",
        "name": "Test Channel",
        "type": "openai",
        "status": "active",
        "models": vec!["gpt-4", "gpt-3.5-turbo"],
        "created_at": chrono::Utc::now().timestamp(),
    });

    c.bench_function("json_serialization", |b| {
        b.iter(|| {
            let _ = serde_json::to_string(&data);
        });
    });
}

/// Benchmark JSON deserialization
fn bench_json_deserialization(c: &mut Criterion) {
    let json_str = r#"{
        "id": "123",
        "name": "Test Channel",
        "type": "openai",
        "status": "active",
        "models": ["gpt-4", "gpt-3.5-turbo"],
        "created_at": 1234567890
    }"#;

    c.bench_function("json_deserialization", |b| {
        b.iter(|| {
            let _data: serde_json::Value = serde_json::from_str(json_str).unwrap();
        });
    });
}

/// Benchmark channel load balancing (round-robin)
fn bench_round_robin_selection(c: &mut Criterion) {
    let channels: Vec<usize> = (0..10).collect();
    let mut current = 0;

    c.bench_function("round_robin_selection", |b| {
        b.iter(|| {
            current = (current + 1) % channels.len();
            black_box(&channels[current]);
        });
    });
}

/// Benchmark channel load balancing (random)
fn bench_random_selection(c: &mut Criterion) {
    let channels: Vec<usize> = (0..10).collect();

    c.bench_function("random_selection", |b| {
        b.iter(|| {
            let idx = rand::random::<usize>() % channels.len();
            black_box(&channels[idx]);
        });
    });
}

/// Benchmark rate limiting check
fn bench_rate_limit_check(c: &mut Criterion) {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    
    let mut rate_limits: Arc<Mutex<HashMap<String, (u32, u128)>>> = Arc::new(Mutex::new(HashMap::new()));
    let requests_per_sec = 100u32;
    let client_id = "test-client".to_string();

    c.bench_function("rate_limit_check", |b| {
        b.iter(|| {
            let now = chrono::Utc::now().timestamp_millis();
            let mut limits = rate_limits.lock().unwrap();
            
            let entry = limits.entry(client_id.clone()).or_insert((0, now));
            
            if now - entry.1 > 1000 {
                entry.0 = 0;
                entry.1 = now;
            }
            
            if entry.0 < requests_per_sec {
                entry.0 += 1;
                black_box(true); // Allowed
            } else {
                black_box(false); // Rate limited
            }
        });
    });
}

criterion_group!(
    benches,
    bench_jwt_encoding,
    bench_jwt_decoding,
    bench_password_hashing,
    bench_password_verification,
    bench_uuid_generation,
    bench_json_serialization,
    bench_json_deserialization,
    bench_round_robin_selection,
    bench_random_selection,
    bench_rate_limit_check,
);

criterion_main!(benches);
