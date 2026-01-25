use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use keyring_cli::crypto::{aes256gcm, argon2id};

fn bench_argon2id_derive_key(c: &mut Criterion) {
    let password = "test-password-for-benchmarking";
    let salt = argon2id::generate_salt();

    c.bench_function("argon2id_derive_key", |b| {
        b.iter(|| argon2id::derive_key(black_box(password), black_box(&salt)).unwrap())
    });
}

fn bench_aes256gcm_encrypt(c: &mut Criterion) {
    let plaintext = b"benchmark-data-12345678";
    let key = [0u8; 32];

    c.bench_function("aes256gcm_encrypt", |b| {
        b.iter(|| aes256gcm::encrypt(black_box(plaintext), black_box(&key)).unwrap())
    });
}

fn bench_aes256gcm_decrypt(c: &mut Criterion) {
    let plaintext = b"benchmark-data-12345678";
    let key = [1u8; 32];
    let (ciphertext, nonce) = aes256gcm::encrypt(plaintext, &key).unwrap();

    c.bench_function("aes256gcm_decrypt", |b| {
        b.iter(|| {
            aes256gcm::decrypt(black_box(&ciphertext), black_box(&nonce), black_box(&key)).unwrap()
        })
    });
}

criterion_group!(
    crypto_benches,
    bench_argon2id_derive_key,
    bench_aes256gcm_encrypt,
    bench_aes256gcm_decrypt
);
criterion_main!(crypto_benches);
