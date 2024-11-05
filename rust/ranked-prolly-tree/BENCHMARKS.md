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
| get random 1 entry             |        100 |     0.0016 |     0.0211 |     0.0090 |   0.0035 |     111439 |
| get random 100 entries         |        100 |     0.7578 |     1.1250 |     0.8804 |   0.0544 |     113583 |
| set random 1 entry             |        100 |     0.0097 |     0.0667 |     0.0389 |   0.0160 |      25692 |
| set random 100 entries         |        100 |     3.4114 |     5.6470 |     3.7969 |   0.2620 |      26337 |
| set random 1k entries          |         10 |    36.7802 |    38.3114 |    37.6431 |   0.5017 |      26565 |


### 50k entries

|                                | iterations |   min (ms) |   max (ms) |   avg (ms) |      std |    ops / s |
| :----------------------------- | ---------: | ---------: | ---------: | ---------: | -------: | ---------: |
| get random 1 entry             |        100 |     0.0052 |     0.0457 |     0.0161 |   0.0082 |      62178 |
| get random 100 entries         |        100 |     1.3878 |     3.4897 |     1.6181 |   0.2155 |      61802 |
| set random 1 entry             |        100 |     0.0168 |     0.2331 |     0.0724 |   0.0428 |      13815 |
| set random 100 entries         |        100 |     5.6635 |     7.9281 |     6.5825 |   0.4239 |      15192 |
| set random 1k entries          |         10 |    63.4977 |    67.3309 |    65.0190 |   1.1920 |      15380 |


### 1m entries

|                                | iterations |   min (ms) |   max (ms) |   avg (ms) |      std |    ops / s |
| :----------------------------- | ---------: | ---------: | ---------: | ---------: | -------: | ---------: |
| get random 1 entry             |        100 |     0.0154 |     0.1477 |     0.0442 |   0.0204 |      22629 |
| get random 100 entries         |        100 |     3.1801 |     4.1777 |     3.6133 |   0.1520 |      27676 |
| set random 1 entry             |        100 |     0.0468 |     0.2684 |     0.1279 |   0.0473 |       7819 |
| set random 100 entries         |        100 |    11.1793 |    14.6534 |    12.5106 |   0.6420 |       7993 |
| set random 1k entries          |         10 |   120.5261 |   131.0766 |   123.4308 |   2.8496 |       8102 |


## wasm32 + IndexedDb

### 1k entries

|                                | iterations |   min (ms) |   max (ms) |   avg (ms) |      std |    ops / s |
| :----------------------------- | ---------: | ---------: | ---------: | ---------: | -------: | ---------: |
| get random 1 entry             |        100 |     0.1700 |     0.8050 |     0.2990 |   0.1168 |       3344 |
| get random 100 entries         |        100 |    19.1650 |    35.4800 |    25.5318 |   3.2603 |       3917 |
| set random 1 entry             |        100 |     0.5100 |     9.7950 |     0.8520 |   0.9134 |       1174 |
| set random 100 entries         |        100 |    64.3650 |    97.2300 |    82.2235 |   6.0826 |       1216 |
| set random 1k entries          |         10 |   815.1150 |   867.8950 |   836.8155 |  16.1293 |       1195 |

### 50k entries
    
|                                | iterations |   min (ms) |   max (ms) |   avg (ms) |      std |    ops / s |
| :----------------------------- | ---------: | ---------: | ---------: | ---------: | -------: | ---------: |
| get random 1 entry             |        100 |     0.2500 |     1.6650 |     0.4158 |   0.1753 |       2405 |
| get random 100 entries         |        100 |    34.7950 |   136.0600 |    41.4974 |  10.3473 |       2410 |
| set random 1 entry             |        100 |     0.7750 |    13.5700 |     1.3923 |   1.4189 |        718 |
| set random 100 entries         |        100 |   106.1150 |   259.4850 |   123.6876 |  15.3566 |        808 |
| set random 1k entries          |         10 |  1172.5850 |  1291.2150 |  1222.1860 |  36.6748 |        818 |

### 1m entries
    
|                                | iterations |   min (ms) |   max (ms) |   avg (ms) |      std |    ops / s |
| :----------------------------- | ---------: | ---------: | ---------: | ---------: | -------: | ---------: |
| get random 1 entry             |        100 |     0.3800 |     2.7250 |     0.5847 |   0.2668 |       1710 |
| get random 100 entries         |        100 |    47.3400 |    70.8950 |    56.9790 |   5.0866 |       1755 |
| set random 1 entry             |        100 |     1.1100 |    13.1000 |     1.8509 |   1.2899 |        540 |
| set random 100 entries         |        100 |   160.4800 |   354.5550 |   179.7098 |  19.5886 |        556 |
| set random 1k entries          |         10 |  1731.9400 |  1907.4200 |  1793.6290 |  44.7622 |        558 |
    
## wasm32 + IndexedDb + LRU Cache(1000)

### 1k entries
    
|                                | iterations |   min (ms) |   max (ms) |   avg (ms) |      std |    ops / s |
| :----------------------------- | ---------: | ---------: | ---------: | ---------: | -------: | ---------: |
| get random 1 entry             |        100 |     0.0000 |     0.2050 |     0.0171 |   0.0203 |      58309 |
| get random 100 entries         |        100 |     0.8850 |     2.7700 |     1.0497 |   0.2122 |      95265 |
| set random 1 entry             |        100 |     0.3650 |     1.5300 |     0.5748 |   0.1630 |       1740 |
| set random 100 entries         |        100 |    44.1850 |    61.6700 |    50.8364 |   3.8369 |       1967 |
| set random 1k entries          |         10 |   497.9450 |   524.2000 |   511.8470 |   7.3116 |       1954 |

### 50k entries
    
|                                | iterations |   min (ms) |   max (ms) |   avg (ms) |      std |    ops / s |
| :----------------------------- | ---------: | ---------: | ---------: | ---------: | -------: | ---------: |
| get random 1 entry             |        100 |     0.0050 |     0.1950 |     0.0228 |   0.0213 |      43860 |
| get random 100 entries         |        100 |     1.5400 |     2.8400 |     1.8338 |   0.2147 |      54533 |
| set random 1 entry             |        100 |     0.4750 |     1.7600 |     0.7104 |   0.1878 |       1408 |
| set random 100 entries         |        100 |    68.3150 |   100.7250 |    78.7852 |   6.7943 |       1269 |
| set random 1k entries          |         10 |   820.6650 |   942.1150 |   850.4615 |  33.4185 |       1176 |
    
  
### 1m entries
    
|                                | iterations |   min (ms) |   max (ms) |   avg (ms) |      std |    ops / s |
| :----------------------------- | ---------: | ---------: | ---------: | ---------: | -------: | ---------: |
| get random 1 entry             |        100 |     0.0250 |     3.1650 |     0.3397 |   0.3061 |       2944 |
| get random 100 entries         |        100 |    22.7500 |    39.5000 |    28.9441 |   3.7628 |       3455 |
| set random 1 entry             |        100 |     0.8000 |     1.6750 |     1.1542 |   0.2264 |        866 |
| set random 100 entries         |        100 |   122.1700 |   240.8250 |   142.9269 |  14.6929 |        700 |
| set random 1k entries          |         10 |  1318.7500 |  1497.7500 |  1368.0100 |  48.6877 |        731 |
    
