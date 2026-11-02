| Delay steps (trace rows) | Work time | Proof generation | Verify time (median) | Proof size |
|--------------------------|-----------|------------------|----------------------|------------|
| 1,024 | 2.41 ms | 91.36 ms | 0.52 ms | 100.4 KB |
| 4,096 | 8.19 ms | 455.05 ms | 0.64 ms | 113.2 KB |
| 16,384 | 35.82 ms | 2.052 s | 0.70 ms | 123.3 KB |
| 65,536 | 145.03 ms | 9.195 s | 0.83 ms | 138.6 KB |

Machine: Intel(R) Xeon(R) Processor, 4 vCPU, 15.6 GiB RAM
rustc: rustc 1.99.1 (58cf8f9c3 2026-10-13)
Options: queries=33 blowup=8 grinding=0 extension=None cells=64
Method: one trace build, one proof, then 19 timed verifications after one warmup. Verify figures are the median.
