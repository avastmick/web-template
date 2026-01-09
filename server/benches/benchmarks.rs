//! Criterion benchmarks for critical server operations.
//!
//! Run with: cargo bench
//! HTML reports generated in: target/criterion/

// Allow unsafe code in benchmarks for environment variable setup
// This is necessary because std::env::set_var is unsafe in Rust 2024 edition
#![allow(unsafe_code)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use server::core::password_utils::{hash_password, verify_password};
use server::services::auth_service::AuthService;
use uuid::Uuid;

/// Benchmark password hashing with Argon2.
///
/// This measures the time to hash passwords of various lengths.
/// Argon2 is intentionally slow to resist brute-force attacks.
fn bench_password_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("password_hashing");

    // Test different password lengths
    let passwords = [
        ("short_12", "Password123!"),
        ("medium_24", "SecurePassword12345678!"),
        ("long_64", "VeryLongAndSecurePasswordThatSomeoneActuallyMightUse12345678!!!!"),
    ];

    for (name, password) in passwords {
        group.bench_with_input(BenchmarkId::new("hash", name), &password, |b, password| {
            b.iter(|| hash_password(black_box(password)))
        });
    }

    group.finish();
}

/// Benchmark password verification with Argon2.
///
/// This measures the time to verify a password against a stored hash.
fn bench_password_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("password_verification");

    // Pre-hash passwords for verification benchmarks
    let test_cases = [
        ("short_12", "Password123!"),
        ("medium_24", "SecurePassword12345678!"),
    ];

    for (name, password) in test_cases {
        let hashed = hash_password(password).expect("Failed to hash password for benchmark setup");

        group.bench_with_input(
            BenchmarkId::new("verify_correct", name),
            &(password, &hashed),
            |b, (password, hashed)| b.iter(|| verify_password(black_box(password), black_box(hashed))),
        );

        // Also benchmark failed verification (wrong password)
        group.bench_with_input(
            BenchmarkId::new("verify_wrong", name),
            &("WrongPassword!", &hashed),
            |b, (wrong_password, hashed)| {
                b.iter(|| verify_password(black_box(wrong_password), black_box(hashed)))
            },
        );
    }

    group.finish();
}

/// Benchmark JWT token generation.
///
/// This measures the time to generate JWT tokens for authentication.
fn bench_jwt_generation(c: &mut Criterion) {
    // Set up environment for AuthService
    // SAFETY: This is benchmark code running single-threaded during setup
    unsafe {
        std::env::set_var("JWT_SECRET", "benchmark-secret-key-for-testing-purposes-only");
    }

    let auth_service = AuthService::new().expect("Failed to create AuthService for benchmark");

    let mut group = c.benchmark_group("jwt_generation");

    // Generate tokens for different user scenarios
    let test_cases = [
        ("simple_email", "user@example.com"),
        ("complex_email", "user.name+tag@subdomain.example.co.uk"),
    ];

    for (name, email) in test_cases {
        let user_id = Uuid::new_v4();

        group.bench_with_input(
            BenchmarkId::new("generate", name),
            &(user_id, email),
            |b, (user_id, email)| b.iter(|| auth_service.generate_token(black_box(*user_id), black_box(email))),
        );
    }

    group.finish();
}

/// Benchmark JWT token validation.
///
/// This measures the time to validate JWT tokens.
fn bench_jwt_validation(c: &mut Criterion) {
    // Set up environment for AuthService
    // SAFETY: This is benchmark code running single-threaded during setup
    unsafe {
        std::env::set_var("JWT_SECRET", "benchmark-secret-key-for-testing-purposes-only");
    }

    let auth_service = AuthService::new().expect("Failed to create AuthService for benchmark");

    let mut group = c.benchmark_group("jwt_validation");

    // Pre-generate tokens for validation benchmarks
    let test_cases = [
        ("simple_email", "user@example.com"),
        ("complex_email", "user.name+tag@subdomain.example.co.uk"),
    ];

    for (name, email) in test_cases {
        let user_id = Uuid::new_v4();
        let token = auth_service
            .generate_token(user_id, email)
            .expect("Failed to generate token for benchmark setup");

        group.bench_with_input(BenchmarkId::new("validate", name), &token, |b, token| {
            b.iter(|| auth_service.validate_token(black_box(token)))
        });
    }

    // Also benchmark validation of an invalid token
    group.bench_function("validate_invalid", |b| {
        b.iter(|| auth_service.validate_token(black_box("invalid.token.here")))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_password_hashing,
    bench_password_verification,
    bench_jwt_generation,
    bench_jwt_validation,
);

criterion_main!(benches);
