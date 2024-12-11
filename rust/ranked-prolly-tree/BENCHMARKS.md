# Benchmarks

A benchmarking suite (`./examples/benchmark.rs`) generates benchmark values for various storage implementations within
the Ranked Prolly Tree (RPT) implementation.
Featuring similar functionality, these values are compared to [Okra's benchmarks](https://github.com/canvasxyz/okra/blob/main/BENCHMARKS.md)
with some notable distinctions:

* RPT writes do not invalidate previous trees.
  * Reading a key Okra directly reads via key from LMDB -- very fast!
* RPT does not have batch transactions.
  * As such, setting/getting X values are essentially linear.
* RPT designed for caching, especially over the wire from a replica.
  * Current benchmarks are single-replica, single-device
* TODO: Measure the overhead of the specific storage (e.g. IndexedDb) to verify the bulk of the costs are due to storage lookups.

## native + in-memory

### 1k entries

|                                | iterations |   min (ms) |   max (ms) |   avg (ms) |      std |    ops / s |
| :----------------------------- | ---------: | ---------: | ---------: | ---------: | -------: | ---------: |
| get random 1 entry             |        100 |     0.0015 |     0.0155 |     0.0088 |   0.0030 |     113419 |
| get random 100 entries         |        100 |     0.7392 |     2.5025 |     0.8676 |   0.1711 |     115265 |
| iterate over all entries       |        100 |     0.1308 |     0.1441 |     0.1338 |   0.0022 |    7472069 |
| set random 1 entry             |        100 |     0.0098 |     0.0612 |     0.0378 |   0.0143 |      26490 |
| set random 100 entries         |        100 |     3.2372 |     4.0399 |     3.6672 |   0.1477 |      27268 |
| set random 1k entries          |         10 |    35.8465 |    38.9687 |    36.9037 |   0.8709 |      27098 |

### 50k entries

|                                | iterations |   min (ms) |   max (ms) |   avg (ms) |      std |    ops / s |
| :----------------------------- | ---------: | ---------: | ---------: | ---------: | -------: | ---------: |
| get random 1 entry             |        100 |     0.0029 |     0.0478 |     0.0154 |   0.0077 |      65137 |
| get random 100 entries         |        100 |     1.2299 |     1.8042 |     1.5197 |   0.1010 |      65802 |
| iterate over all entries       |        100 |     6.6780 |     8.4948 |     6.7718 |   0.1803 |    7383522 |
| set random 1 entry             |        100 |     0.0158 |     0.1950 |     0.0660 |   0.0384 |      15160 |
| set random 100 entries         |        100 |     5.5090 |     8.4409 |     6.3486 |   0.3811 |      15751 |
| set random 1k entries          |         10 |    61.9223 |    65.3108 |    63.6006 |   0.8722 |      15723 |

### 1m entries

|                                | iterations |   min (ms) |   max (ms) |   avg (ms) |      std |    ops / s |
| :----------------------------- | ---------: | ---------: | ---------: | ---------: | -------: | ---------: |
| get random 1 entry             |        100 |     0.0140 |     0.0823 |     0.0370 |   0.0128 |      27051 |
| get random 100 entries         |        100 |     3.2441 |     4.3235 |     3.6146 |   0.1856 |      27666 |
| iterate over all entries       |        100 |   180.1743 |   198.8865 |   184.3576 |   2.9930 |    5424242 |
| set random 1 entry             |        100 |     0.0430 |     0.2889 |     0.1281 |   0.0480 |       7808 |
| set random 100 entries         |        100 |    11.1947 |    14.6874 |    12.4183 |   0.6094 |       8053 |
| set random 1k entries          |         10 |   119.7858 |   205.0411 |   132.0660 |  25.1173 |       7572 |

## wasm32 + IndexedDb + LRU Cache(10000)

### 1k entries

|                                | iterations |   min (ms) |   max (ms) |   avg (ms) |      std |    ops / s |
| :----------------------------- | ---------: | ---------: | ---------: | ---------: | -------: | ---------: |
| get random 1 entry             |        100 |     0.0000 |     0.2150 |     0.0176 |   0.0215 |      56657 |
| get random 100 entries         |        100 |     0.9250 |     1.3300 |     1.0975 |   0.0762 |      91116 |
| iterate over all entries       |        100 |     0.1400 |     0.4600 |     0.1588 |   0.0313 |    6297229 |
| set random 1 entry             |        100 |     0.3350 |     1.8600 |     0.6261 |   0.2411 |       1597 |
| set random 100 entries         |        100 |    44.9050 |    73.0500 |    55.5719 |   5.2910 |       1799 |
| set random 1k entries          |         10 |   525.8600 |   615.5000 |   559.2845 |  23.2049 |       1788 |
    
### 50k entries
    
|                                | iterations |   min (ms) |   max (ms) |   avg (ms) |      std |    ops / s |
| :----------------------------- | ---------: | ---------: | ---------: | ---------: | -------: | ---------: |
| get random 1 entry             |        100 |     0.0050 |     0.0550 |     0.0182 |   0.0123 |      54795 |
| get random 100 entries         |        100 |     1.4500 |     3.4050 |     1.8537 |   0.2098 |      53945 |
| iterate over all entries       |        100 |     7.4800 |     9.3150 |     7.8253 |   0.1963 |     900966 |
| set random 1 entry             |        100 |     0.4350 |     2.2000 |     0.7246 |   0.2181 |       1380 |
| set random 100 entries         |        100 |    66.9400 |   146.8950 |    78.3272 |   8.7690 |       1277 |
| set random 1k entries          |         10 |   745.9150 |   799.0750 |   776.4795 |  16.3981 |       1288 |
    
    
### 1m entries
    
|                                | iterations |   min (ms) |   max (ms) |   avg (ms) |      std |    ops / s |
| :----------------------------- | ---------: | ---------: | ---------: | ---------: | -------: | ---------: |
| get random 1 entry             |        100 |     0.0200 |     1.9550 |     0.1638 |   0.2053 |       6103 |
| get random 100 entries         |        100 |     7.8050 |    21.9250 |    11.1967 |   2.1361 |       8931 |
| iterate over all entries       |        100 |  2241.7300 |  3042.8600 |  2697.4486 | 105.5341 |       4507 |
| set random 1 entry             |        100 |     0.7600 |     6.7150 |     1.3836 |   0.6509 |        723 |
| set random 100 entries         |        100 |   108.9800 |   148.1600 |   126.7325 |   8.2948 |        789 |
| set random 1k entries          |         10 |  1184.9750 |  1305.9200 |  1217.7345 |  31.6706 |        821 |
