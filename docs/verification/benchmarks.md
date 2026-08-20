# Benchmarks

> **Status:** Repository-wide performance measurement contract.

A benchmark is an experiment that measures a defined property under defined conditions. It is evidence about those conditions, not a universal statement about NROS.

## 1. Benchmark record

A reproducible benchmark record SHOULD contain:

```text
Benchmark ID
Claim / criterion
Repository revision
Binary / artifact
Hardware
Operating system / kernel
Rust toolchain
Target triple
Features / profile
Workload / dataset
Input distribution
Warm-up policy
Measurement method
Sample count
Observed statistics
Artifacts / logs
Limitations
```

The minimum required context depends on the measurement and claim.

## 2. Measurement chain

```text
Workload
   ↓
Environment
   ↓
Measurement method
   ↓
Samples
   ↓
Statistics
   ↓
Observed result
   ↓
Interpretation
   ↓
Limitations
```

Changing a material input in this chain can invalidate comparison with an earlier result.

## 3. What to report

Where meaningful, report more than a single average. Depending on the metric, include:

- minimum;
- maximum;
- mean;
- median;
- percentile values;
- standard deviation or other variability measure;
- sample count;
- outliers/anomalies;
- allocation counts;
- throughput;
- latency;
- jitter.

The chosen statistics MUST match the claim being evaluated.

## 4. Latency claims

Latency documentation should distinguish:

```text
Target latency
Observed samples
Typical latency
Maximum observed latency
Statistical percentile
Worst-case bound
```

For example:

```text
p99 < 1 ms
        ≠
max observed < 1 ms
        ≠
provable worst-case < 1 ms
```

A benchmark cannot establish a mathematical worst-case bound merely by increasing the sample count.

## 5. Throughput claims

Throughput measurements MUST identify the workload and resource conditions that generated them.

```text
messages / second
```

without message size, payload characteristics, concurrency, transport, hardware, and measurement method is incomplete evidence for a performance claim.

## 6. Allocation / zero-copy claims

Allocation measurements should identify what part of the system was measured.

```text
One internal operation allocates zero times
        ≠
Complete end-to-end path allocates zero times
```

Likewise, the presence of a zero-copy data structure is not sufficient evidence that all surrounding serialization, transport, logging, or application layers avoid copies.

## 7. Real-time and determinism

Benchmarks can provide timing evidence, but real-time claims require stronger analysis.

```text
Benchmark sample
      ↓
Distribution
      ↓
Observed maximum
      ↓
Bound analysis
      ↓
Target qualification
```

Similarly, repeated identical benchmark output does not by itself prove deterministic behavior across all executions and environments.

## 8. Comparison rules

Comparing benchmark results is meaningful only when material conditions are controlled or explicitly normalized.

A comparison SHOULD preserve:

- same revision or documented code difference;
- same workload;
- same measurement method;
- same hardware class;
- same toolchain/profile;
- same feature set;
- same relevant configuration.

If conditions differ, the comparison must state the difference.

## 9. Reproducibility

A published benchmark should provide enough information to repeat the measurement.

At minimum:

```text
revision
command
configuration
workload
environment
result
artifact/log location
```

If the benchmark cannot currently be reproduced, label it accordingly rather than presenting it as current verified evidence.

## 10. Benchmark status

Use precise states:

- **Measured** — execution produced the reported measurement;
- **Reproducible** — independent/repeated execution reproduced the relevant result under defined conditions;
- **Partially verified** — only part of the performance criterion is established;
- **Not verified** — insufficient evidence;
- **Blocked** — measurement could not execute;
- **Stale** — measurement no longer applies to the relevant revision/environment.

## 11. Benchmark anti-patterns

Avoid:

- reporting a number without its environment;
- comparing unlike workloads;
- using average latency as a worst-case guarantee;
- treating a microbenchmark as an end-to-end measurement;
- hiding warm-up or failed samples;
- silently changing compiler/profile settings;
- claiming zero allocations from an uninstrumented path;
- claiming deterministic timing from a small sample;
- publishing stale results as current.

## 12. Claim linkage

Every significant benchmark should identify the claim or criterion it supports:

```text
Benchmark
   ↓
Claim ID
   ↓
Criterion
   ↓
Observed result
   ↓
Conclusion
   ↓
Limitations
```

A benchmark without a defined claim can still be useful exploratory data, but it should not automatically be treated as verification evidence.

## 13. Related documentation

- [Verification Overview](README.md)
- [Evidence Model](evidence-model.md)
- [Claims](claims.md)
- [Test Strategy](test-strategy.md)
- [Validation](validation.md)
- [Reference](../reference/README.md)
