//! Benchmarks for Triple-Layer Encryption
//!
//! Performance tests to ensure we meet targets:
//! - Encryption: >10 MB/sec
//! - Decryption: >10 MB/sec
//! - Key derivation: <1 second

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use chakchat_crypto::encryption::{TripleLayerEncryption, KEY_SIZE};
use chakchat_crypto::key_exchange::KeyPair;
use chakchat_crypto::post_quantum::PostQuantumKeyPair;

fn benchmark_triple_layer_encryption(c: &mut Criterion) {
    let shared_secret = black_box([42u8; KEY_SIZE]);

    c.bench_function("triple_layer_encrypt_1kb", |b| {
        b.iter(|| {
            let mut enc = TripleLayerEncryption::new(&shared_secret).unwrap();
            let plaintext = black_box(vec![0u8; 1024]);
            enc.encrypt(&plaintext).unwrap()
        })
    });

    c.bench_function("triple_layer_encrypt_10kb", |b| {
        b.iter(|| {
            let mut enc = TripleLayerEncryption::new(&shared_secret).unwrap();
            let plaintext = black_box(vec![0u8; 10 * 1024]);
            enc.encrypt(&plaintext).unwrap()
        })
    });

    c.bench_function("triple_layer_encrypt_100kb", |b| {
        b.iter(|| {
            let mut enc = TripleLayerEncryption::new(&shared_secret).unwrap();
            let plaintext = black_box(vec![0u8; 100 * 1024]);
            enc.encrypt(&plaintext).unwrap()
        })
    });
}

fn benchmark_decryption(c: &mut Criterion) {
    let shared_secret = black_box([42u8; KEY_SIZE]);

    c.bench_function("triple_layer_decrypt_1kb", |b| {
        b.iter_batched(
            || {
                let mut enc = TripleLayerEncryption::new(&shared_secret).unwrap();
                let plaintext = vec![0u8; 1024];
                enc.encrypt(&plaintext).unwrap()
            },
            |encrypted| {
                let mut dec = TripleLayerEncryption::new(&shared_secret).unwrap();
                dec.decrypt(&encrypted).unwrap()
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn benchmark_key_agreement(c: &mut Criterion) {
    c.bench_function("ecdh_key_agreement", |b| {
        b.iter_batched(
            || {
                let kp1 = black_box(KeyPair::generate().unwrap());
                let kp2 = black_box(KeyPair::generate().unwrap());
                (kp1, kp2)
            },
            |(kp1, kp2)| {
                let secret1 = kp1.compute_shared_secret(&kp2.public_key).unwrap();
                let secret2 = kp2.compute_shared_secret(&kp1.public_key).unwrap();
                assert_eq!(secret1, secret2);
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn benchmark_post_quantum(c: &mut Criterion) {
    c.bench_function("kyber1024_keypair_generation", |b| {
        b.iter(|| {
            PostQuantumKeyPair::generate().unwrap()
        })
    });

    c.bench_function("kyber1024_encapsulate", |b| {
        b.iter_batched(
            || {
                let kp = black_box(PostQuantumKeyPair::generate().unwrap());
                kp
            },
            |kp| {
                PostQuantumKeyPair::encapsulate(kp.public_key_bytes()).unwrap()
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    benchmark_triple_layer_encryption,
    benchmark_decryption,
    benchmark_key_agreement,
    benchmark_post_quantum
);
criterion_main!(benches);
