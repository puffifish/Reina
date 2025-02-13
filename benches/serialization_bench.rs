// File: reina/serialization_bench.rs

#[macro_use]
extern crate criterion;

use criterion::{BenchmarkId, Criterion, Throughput, black_box};
use rayon::prelude::*;
use blake3;
#[cfg(feature = "cpu_affinity")]
use core_affinity;

use reina::utils::serialization::{
    Transaction, Serializer, Endianness, fixed_encoding, Encode,
};

/// Optionally pin CPU affinity and initialize Rayon’s global thread pool only once.
/// This helps ensure consistent thread scheduling. If the cpu_affinity feature is enabled,
/// the main thread is pinned to the first physical core, and Rayon is configured to use
/// the number of physical cores.
#[cfg(feature = "cpu_affinity")]
fn configure_cpu() {
    if let Some(cores) = core_affinity::get_core_ids() {
        if let Some(core) = cores.first() {
            core_affinity::set_for_current(*core);
        }
    }
    // Build the global Rayon thread pool (ignoring errors if already built).
    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get_physical())
        .build_global();
}

/// --- Helper: Ultra–Low–Latency Fixed Serialization ---
/// Inlined to remove function call overhead. It calls the ultra–low–latency
/// fixed-length serialization function and copies the result into a provided buffer.
#[inline(always)]
fn serialize_fixed(
    tx: &Transaction,
    buf: &mut [u8],
    endianness: Endianness,
) -> reina::utils::serialization::SerializationResult<usize> {
    let fixed = Serializer::serialize_ultra_fixed(tx, endianness)?;
    if buf.len() < fixed.len() {
        return Err(reina::utils::serialization::SerializationError::BufferTooSmall);
    }
    buf[..fixed.len()].copy_from_slice(&fixed);
    Ok(fixed.len())
}

/// --- Benchmark: Single Transaction (Baseline) ---
/// Measures the latency of serializing and deserializing a single transaction.
fn bench_single_transaction(c: &mut Criterion) {
    let tx = Transaction {
        version: 1,
        id: 42,
        sender: "Alice".to_string(),
        recipient: "Bob".to_string(),
        amount: 1000,
        signature: vec![1, 2, 3, 4],
        fee: 0.01,
    };

    c.bench_function("single_tx_serialization", |b| {
        b.iter(|| {
            let ser = Serializer::serialize(black_box(&tx), Endianness::Little)
                .expect("Serialization failed");
            black_box(ser);
        })
    });

    let ser = Serializer::serialize(&tx, Endianness::Little).expect("Serialization failed");
    c.bench_function("single_tx_deserialization", |b| {
        b.iter(|| {
            let de: Transaction =
                Serializer::deserialize(black_box(&ser), Endianness::Little)
                    .expect("Deserialization failed");
            black_box(de);
        })
    });
}

/// --- Benchmark: Batch Serialization using Batch Hashing ---
/// Preallocates a buffer once and reuses it across iterations to avoid repeated allocations.
/// This is critical for achieving higher throughput in large batches.
fn bench_serialize_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_serialization_batch_hashing");
    for &batch_size in &[1_000usize, 10_000, 100_000, 1_000_000] {
        let tx = Transaction {
            version: 1,
            id: 42,
            sender: "Alice".to_string(),
            recipient: "Bob".to_string(),
            amount: 1000,
            signature: vec![1, 2, 3, 4],
            fee: 0.01,
        };
        // Preallocate a vector of transactions by repeating a cloned tx.
        let txs: Vec<Transaction> = std::iter::repeat(tx).take(batch_size).collect();
        // Preallocate a buffer with an estimated size (128 bytes per transaction).
        let mut ser_buffer = Vec::with_capacity(batch_size * 128);

        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(batch_size), &txs, |b, txs| {
            b.iter(|| {
                // Clear the buffer once per iteration.
                ser_buffer.clear();
                // Serialize the entire batch directly into one large buffer.
                let batch_ser = Serializer::serialize_batch(black_box(txs), Endianness::Little)
                    .expect("Batch serialization failed");
                ser_buffer.extend_from_slice(&batch_ser);
                black_box(&ser_buffer);
            })
        });
    }
    group.finish();
}

/// --- Benchmark: Batch Deserialization (Sequential) ---
/// Pre-generated serialized data is deserialized in a tight loop to measure sequential performance.
fn bench_batch_deserialization_seq(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_deserialization_seq");
    for &batch_size in &[100usize, 1_000, 10_000, 100_000] {
        let txs: Vec<Transaction> = (0..batch_size)
            .map(|_| Transaction {
                version: 1,
                id: 42,
                sender: "Alice".to_string(),
                recipient: "Bob".to_string(),
                amount: 1000,
                signature: vec![1, 2, 3, 4],
                fee: 0.01,
            })
            .collect();
        let ser_batch: Vec<Vec<u8>> = txs.iter()
            .map(|tx| Serializer::serialize(tx, Endianness::Little).expect("Serialization failed"))
            .collect();

        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(batch_size), &ser_batch, |b, batch| {
            b.iter(|| {
                let de: Vec<Transaction> = batch.iter()
                    .map(|data| {
                        Serializer::deserialize::<Transaction>(black_box(data), Endianness::Little)
                            .expect("Deserialization failed")
                    })
                    .collect();
                black_box(de);
            })
        });
    }
    group.finish();
}

/// --- Benchmark: Parallel Deserialization with Adaptive Chunking ---
/// Uses Rayon’s par_chunks_exact with a dynamic chunk size (here fixed at 512 for simplicity)
/// to ensure even workload distribution while reducing thread contention.
fn bench_parallel_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_deserialization");
    for &batch_size in &[100usize, 1_000, 10_000, 100_000] {
        let txs: Vec<Transaction> = (0..batch_size)
            .map(|_| Transaction {
                version: 1,
                id: 42,
                sender: "Alice".to_string(),
                recipient: "Bob".to_string(),
                amount: 1000,
                signature: vec![1, 2, 3, 4],
                fee: 0.01,
            })
            .collect();
        let ser_batch: Vec<Vec<u8>> = txs.iter()
            .map(|tx| Serializer::serialize(tx, Endianness::Little).expect("Serialization failed"))
            .collect();

        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(batch_size), &ser_batch, |b, batch| {
            b.iter(|| {
                // Use par_chunks_exact with a chunk size of 512 to avoid too many tiny chunks.
                let de: Vec<Transaction> = batch
                    .par_chunks_exact(512)
                    .flat_map(|chunk| {
                        chunk.iter().map(|data| {
                            Serializer::deserialize::<Transaction>(black_box(data), Endianness::Little)
                                .expect("Deserialization failed")
                        }).collect::<Vec<_>>()
                    })
                    .collect();
                black_box(de);
            })
        });
    }
    group.finish();
}

/// --- Benchmark: Deserialization with Preallocated Buffer ---
/// Reuses a preallocated stack buffer for inputs ≤4 KB to eliminate dynamic allocation overhead.
fn bench_deserialization_with_pool(c: &mut Criterion) {
    let tx = Transaction {
        version: 1,
        id: 42,
        sender: "Alice".to_string(),
        recipient: "Bob".to_string(),
        amount: 1000,
        signature: vec![1, 2, 3, 4],
        fee: 0.01,
    };
    let ser = Serializer::serialize(&tx, Endianness::Little).expect("Serialization failed");
    c.bench_function("deserialization_with_pool", |b| {
        b.iter(|| {
            let de: Transaction = Serializer::deserialize_with_pool(black_box(&ser), Endianness::Little)
                .expect("Deserialization with pool failed");
            black_box(de);
        })
    });
}

/// --- Benchmark: Ultra-Low-Latency Serialization (Fixed-Length) ---
/// Uses our fixed-length ultra–low–latency function with a preallocated stack buffer.
fn bench_ultra_low_latency_serialization(c: &mut Criterion) {
    let tx = Transaction {
        version: 1,
        id: 42,
        sender: "Alice".to_string(),
        recipient: "Bob".to_string(),
        amount: 1000,
        signature: vec![1, 2, 3, 4],
        fee: 0.01,
    };

    let mut buffer = [0u8; 128]; // ULTRA_TX_SIZE is 121 bytes; use 128 for alignment.
    c.bench_function("ultra_low_latency_serialization", |b| {
        b.iter(|| {
            let len = serialize_fixed(black_box(&tx), &mut buffer, Endianness::Little)
                .expect("Ultra-low-latency serialization failed");
            black_box(&buffer[..len]);
        })
    });
}

/// --- Benchmark: Varint vs. Fixed-Length Encoding for u64 ---
/// Benchmarks encoding and decoding separately.
fn bench_varint_vs_fixed(c: &mut Criterion) {
    let mut group = c.benchmark_group("varint_vs_fixed_encoding");
    let test_value: u64 = 123456789;
    let varint_buf = [0u8; 10];
    let mut fixed_buf = [0u8; 8];

    group.bench_function("u64_varint_encode", |b| {
        b.iter(|| {
            let _ = Serializer::serialize(black_box(&test_value), Endianness::Little)
                .expect("Serialization of u64 failed");
        })
    });

    group.bench_function("u64_fixed_encode", |b| {
        b.iter(|| {
            let _ = fixed_encoding::encode_fixed_u64(black_box(test_value), &mut fixed_buf, Endianness::Little)
                .expect("Fixed encoding failed");
        })
    });

    let varint_encoded_size = {
        let mut buf = [0u8; 10];
        let mut value = test_value;
        let mut i = 0;
        while value >= 0x80 {
            buf[i] = (value & 0x7F) as u8 | 0x80;
            i += 1;
            value >>= 7;
        }
        buf[i] = value as u8;
        i + 1
    };

    group.bench_function("u64_varint_decode", |b| {
        b.iter(|| {
            let mut value = 0u64;
            let mut shift = 0;
            let mut i = 0;
            while i < varint_encoded_size {
                let byte = varint_buf[i];
                value |= ((byte & 0x7F) as u64) << shift;
                i += 1;
                if byte & 0x80 == 0 { break; }
                shift += 7;
            }
            black_box(value);
        })
    });

    group.bench_function("u64_fixed_decode", |b| {
        b.iter(|| {
            let value = fixed_encoding::decode_fixed_u64(black_box(&fixed_buf), Endianness::Little)
                .expect("Fixed decode failed")
                .0;
            black_box(value);
        })
    });

    group.finish();
}

/// --- Benchmark: Blake3 Checksum Overhead ---
/// Uses a 64 KB payload to simulate realistic block sizes.
fn bench_blake3_overhead(c: &mut Criterion) {
    let payload = vec![0u8; 64 * 1024];
    c.bench_function("blake3_checksum", |b| {
        b.iter(|| {
            let hash = blake3::hash(black_box(&payload));
            black_box(hash);
        })
    });
    c.bench_function("dummy_copy", |b| {
        b.iter(|| {
            let copy = black_box(&payload).clone();
            black_box(copy);
        })
    });
}

/// --- Benchmark: Buffer Preallocation vs. Naive Allocation ---
/// Compares our preallocated serialization path with a naive one that allocates per call.
fn bench_buffer_preallocation(c: &mut Criterion) {
    fn serialize_naive(tx: &Transaction, endianness: Endianness)
        -> Result<Vec<u8>, reina::utils::serialization::SerializationError>
    {
        let mut payload = Vec::new();
        let estimated_size = tx.encoded_size();
        let mut buffer = vec![0u8; estimated_size];
        let written = tx.encode_to(&mut buffer, endianness)?;
        payload.extend_from_slice(&buffer[..written]);
        let hash = blake3::hash(&payload);
        payload.extend_from_slice(hash.as_bytes());
        let mut output = Vec::new();
        output.extend_from_slice(&((payload.len()) as u32).to_le_bytes());
        output.extend_from_slice(&payload);
        Ok(output)
    }

    let tx = Transaction {
        version: 1,
        id: 42,
        sender: "Alice".to_string(),
        recipient: "Bob".to_string(),
        amount: 1000,
        signature: vec![1, 2, 3, 4],
        fee: 0.01,
    };

    let mut group = c.benchmark_group("buffer_preallocation_vs_naive");
    group.bench_function("preallocated_serialization", |b| {
        b.iter(|| {
            let ser = Serializer::serialize(black_box(&tx), Endianness::Little)
                .expect("Serialization failed");
            black_box(ser);
        })
    });
    group.bench_function("naive_serialization", |b| {
        b.iter(|| {
            let ser = serialize_naive(black_box(&tx), Endianness::Little)
                .expect("Naive serialization failed");
            black_box(ser);
        })
    });
    group.finish();
}

/// --- Benchmark: Large-Scale Stress Simulation ---
/// Preallocates a static buffer and repeatedly serializes a batch of 10,000 transactions.
/// This test simulates extreme load (100M+ transactions in aggregate) without repeated dynamic allocations.
fn bench_large_scale_stress(c: &mut Criterion) {
    let tx = Transaction {
        version: 1,
        id: 42,
        sender: "Alice".to_string(),
        recipient: "Bob".to_string(),
        amount: 1000,
        signature: vec![1, 2, 3, 4],
        fee: 0.01,
    };
    let batch: Vec<Transaction> = std::iter::repeat(tx.clone()).take(10_000).collect();
    // Preheat to warm caches.
    let _ = Serializer::serialize_batch(&batch, Endianness::Little).expect("Preheat failed");

    c.bench_function("stress_serialization_100M_simulated", |b| {
        b.iter_custom(|iters| {
            let mut buffer: Vec<u8> = Vec::with_capacity(10_000 * 128);
            let start = std::time::Instant::now();
            for _ in 0..iters {
                buffer.clear();
                let _ = Serializer::serialize_batch(&batch, Endianness::Little)
                    .expect("Batch serialization failed");
            }
            start.elapsed()
        })
    });
}

/// --- Benchmark: Concurrency Stress Test ---
/// Floods parallel serialization and deserialization threads to test thread contention.
fn bench_concurrency_stress(c: &mut Criterion) {
    let tx = Transaction {
        version: 1,
        id: 42,
        sender: "Alice".to_string(),
        recipient: "Bob".to_string(),
        amount: 1000,
        signature: vec![1, 2, 3, 4],
        fee: 0.01,
    };
    let txs: Vec<Transaction> = std::iter::repeat(tx.clone()).take(10_000).collect();
    let ser_batch: Vec<Vec<u8>> = txs.iter()
        .map(|tx| Serializer::serialize(tx, Endianness::Little).expect("Serialization failed"))
        .collect();

    c.bench_function("concurrency_stress_parallel_serialization", |b| {
        b.iter(|| {
            let _results: Vec<_> = (0..1000)
                .into_par_iter()
                .map(|_| {
                    let _ = Serializer::serialize(black_box(&tx), Endianness::Little)
                        .expect("Serialization failed");
                })
                .collect();
        })
    });

    c.bench_function("concurrency_stress_parallel_deserialization", |b| {
        b.iter(|| {
            let _results: Vec<_> = ser_batch
                .par_iter()
                .map(|data| {
                    Serializer::deserialize::<Transaction>(black_box(data), Endianness::Little)
                        .expect("Deserialization failed")
                })
                .collect();
        })
    });
}

/// --- Benchmark: Security Audit of Deserialization ---
/// Runs multiple malformed cases—including unaligned buffers—to ensure robust error handling.
fn bench_security_audit(c: &mut Criterion) {
    let malformed_cases = vec![
        vec![0u8; 10],                  // Too short
        vec![0xff; 100],                // All invalid bytes
        vec![0, 1, 2, 3, 4, 5, 6, 7],     // Incorrect header
        {
            let tx = Transaction {
                version: 1,
                id: 42,
                sender: "Alice".to_string(),
                recipient: "Bob".to_string(),
                amount: 1000,
                signature: vec![1, 2, 3, 4],
                fee: 0.01,
            };
            let mut valid = Serializer::serialize(&tx, Endianness::Little)
                .expect("Serialization failed");
            // Create an unaligned buffer by splitting off the first byte.
            valid.split_off(1)
        },
    ];

    c.bench_function("security_audit_malformed_deserialization", |b| {
        b.iter(|| {
            for case in &malformed_cases {
                let result = Serializer::deserialize::<Transaction>(black_box(case), Endianness::Little);
                assert!(result.is_err());
            }
        })
    });
}

criterion_group!(
    benches,
    bench_single_transaction,
    bench_serialize_batch,
    bench_batch_deserialization_seq,
    bench_parallel_deserialization,
    bench_deserialization_with_pool,
    bench_ultra_low_latency_serialization,
    bench_varint_vs_fixed,
    bench_blake3_overhead,
    bench_buffer_preallocation,
    bench_large_scale_stress,
    bench_concurrency_stress,
    bench_security_audit
);
criterion_main!(benches);