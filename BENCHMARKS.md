# FHIRPath Benchmark Results

Last updated: Fri Aug 08 19:24:43 UTC 2025

## Performance Overview

This document contains the latest benchmark results for the FHIRPath implementation.

### Core Components

The following components have been benchmarked with their current performance metrics:

#### Arc_bundle_operations

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Medium | 9.86 ns | 101.44M ops/sec | 9.84 ns |
| Small | 9.88 ns | 101.18M ops/sec | 9.86 ns |
| Medium | 18.29 ms | 54.69 ops/sec | 18.06 ms |
| Small | 2.68 ms | 373.13 ops/sec | 2.61 ms |
| Medium | 43.89 ns | 22.79M ops/sec | 42.17 ns |
| Small | 41.69 ns | 23.99M ops/sec | 41.19 ns |
| Small | 2.93 ms | 341.72 ops/sec | 2.54 ms |
| Small | 92.08 ns | 10.86M ops/sec | 90.24 ns |
| Medium | 93.58 μs | 10.69K ops/sec | 92.32 μs |
| Small | 13.19 μs | 75.84K ops/sec | 12.94 μs |

#### Base64_encoding

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| 100 | 239.91 ns | 4.17M ops/sec | 239.59 ns |
| 1000 | 1.07 μs | 936.60K ops/sec | 1.01 μs |
| 10000 | 8.39 μs | 119.15K ops/sec | 8.37 μs |
| 100000 | 86.57 μs | 11.55K ops/sec | 86.06 μs |
| 100 | 196.06 ns | 5.10M ops/sec | 194.59 ns |
| 1000 | 669.58 ns | 1.49M ops/sec | 669.64 ns |
| 10000 | 5.16 μs | 193.69K ops/sec | 5.14 μs |
| 100000 | 53.95 μs | 18.53K ops/sec | 53.04 μs |

#### Complex_expressions

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Complex | 137.77 ns | 7.26M ops/sec | 132.19 ns |
| Medium | 144.57 ns | 6.92M ops/sec | 134.24 ns |
| Simple | 120.28 ns | 8.31M ops/sec | 108.71 ns |
| Very_complex | 157.28 ns | 6.36M ops/sec | 155.58 ns |
| Complex | 1.50 μs | 666.24K ops/sec | 1.49 μs |
| Medium | 1.24 μs | 805.38K ops/sec | 623.28 ns |
| Simple | 120.35 ns | 8.31M ops/sec | 119.46 ns |
| Very_complex | 2.85 μs | 350.55K ops/sec | 2.75 μs |

#### Criterion

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| With_admission_policy | 357.14 μs | 2.80K ops/sec | 355.54 μs |
| Without_admission_policy | 252.90 μs | 3.95K ops/sec | 246.51 μs |
| New_arena_per_parse | 375.23 ns | 2.67M ops/sec | 375.99 ns |
| Reused_arena_no_reset | 254.29 ns | 3.93M ops/sec | 253.22 ns |
| Reused_arena_with_reset | 256.05 ns | 3.91M ops/sec | 254.04 ns |
| Large_simple_bundle_traversal | 124.01 ms | 8.06 ops/sec | 123.28 ms |
| Medium_bundle_patient_names | 55.58 ms | 17.99 ops/sec | 55.11 ms |
| Medium_bundle_resource_filter | 52.55 ms | 19.03 ops/sec | 51.97 ms |
| Medium_complex_bundle_filter | 61.17 ms | 16.35 ops/sec | 57.19 ms |
| Medium_deep_bundle_traversal | 56.19 ms | 17.80 ops/sec | 55.76 ms |
| Medium_simple_bundle_traversal | 36.12 ms | 27.68 ops/sec | 35.60 ms |
| Small_bundle_patient_names | 9.25 ms | 108.07 ops/sec | 8.65 ms |
| Small_bundle_resource_filter | 7.50 ms | 133.38 ops/sec | 7.57 ms |
| Small_complex_bundle_filter | 8.08 ms | 123.79 ops/sec | 8.12 ms |
| Small_deep_bundle_traversal | 8.06 ms | 124.05 ops/sec | 7.94 ms |
| Small_simple_bundle_traversal | 5.34 ms | 187.32 ops/sec | 5.20 ms |
| High_hit_rate | 124.97 μs | 8.00K ops/sec | 123.39 μs |
| Low_hit_rate | 294.01 μs | 3.40K ops/sec | 282.60 μs |
| Evaluation_context_child_creation | 118.82 ns | 8.42M ops/sec | 118.73 ns |
| Evaluation_context_variable_get | 6.08 ns | 164.45M ops/sec | 6.06 ns |
| Evaluation_context_variable_set | 24.90 ns | 40.15M ops/sec | 22.77 ns |
| Shared_context_child_creation | 53.32 ns | 18.76M ops/sec | 53.12 ns |
| Shared_context_variable_get | 17.79 ns | 56.22M ops/sec | 17.74 ns |
| Shared_context_variable_set | 23.57 ns | 42.43M ops/sec | 22.99 ns |
| Collect_all_inherited_variables | 4.14 μs | 241.43K ops/sec | 3.97 μs |
| Inheritance_chain_creation | 67.74 ns | 14.76M ops/sec | 67.45 ns |
| Inherited_variable_lookup | 67.30 ns | 14.86M ops/sec | 67.00 ns |
| Inherited_variable_shadowing_lookup | 66.57 ns | 15.02M ops/sec | 66.31 ns |
| Cache_new_closure | 61.68 ns | 16.21M ops/sec | 61.42 ns |
| Execute_cached_closure | 34.03 ns | 29.38M ops/sec | 33.86 ns |
| Execute_closures_batch | 212.52 ns | 4.71M ops/sec | 211.92 ns |
| Optimizer_stats | 18.73 ns | 53.39M ops/sec | 18.71 ns |
| Arena_sequential_new_arena | 981.56 ns | 1.02M ops/sec | 958.62 ns |
| Arena_sequential_reset_arena | 634.26 ns | 1.58M ops/sec | 581.75 ns |
| Arena_sequential_reused_arena | 701.82 ns | 1.42M ops/sec | 648.62 ns |
| Best_case_uri | 101.79 μs | 9.82K ops/sec | 100.81 μs |
| Html_entities | 83.18 μs | 12.02K ops/sec | 75.90 μs |
| Large_base64 | 26.60 μs | 37.60K ops/sec | 25.78 μs |
| Mixed_uri | 46.81 μs | 21.36K ops/sec | 45.42 μs |
| Traditional_sequential | 3.33 μs | 299.93K ops/sec | 2.99 μs |
| Worst_case_uri | 47.80 μs | 20.92K ops/sec | 40.83 μs |
| Base64 | 7.20 μs | 138.84K ops/sec | 7.03 μs |
| Hex | 51.49 μs | 19.42K ops/sec | 49.30 μs |
| Html | 30.30 μs | 33.00K ops/sec | 29.85 μs |
| Uri | 52.79 μs | 18.94K ops/sec | 51.64 μs |
| Urlbase64 | 7.18 μs | 139.37K ops/sec | 7.16 μs |
| Basic_creation | 82.51 ns | 12.12M ops/sec | 82.22 ns |
| Builder_creation | 108.35 ns | 9.23M ops/sec | 108.09 ns |
| Child_with_inherited_variables | 48.38 ns | 20.67M ops/sec | 48.03 ns |
| Child_with_input | 23.57 ns | 42.43M ops/sec | 23.50 ns |
| Child_with_shared_variables | 23.61 ns | 42.36M ops/sec | 23.53 ns |
| String_interning_baseline | 27.02 μs | 37.01K ops/sec | 26.88 μs |
| String_interning_hit_rate | 42.69 μs | 23.43K ops/sec | 42.52 μs |
| Hit_rate_test_aggressive_cleanup | 263.82 μs | 3.79K ops/sec | 263.72 μs |
| Hit_rate_test_default | 263.64 μs | 3.79K ops/sec | 262.95 μs |
| Hit_rate_test_disabled | 32.95 μs | 30.35K ops/sec | 32.86 μs |
| Memory_retention_aggressive_cleanup | 432.76 μs | 2.31K ops/sec | 430.19 μs |
| Memory_retention_default | 389.91 μs | 2.56K ops/sec | 388.65 μs |
| Memory_retention_disabled | 65.02 μs | 15.38K ops/sec | 64.59 μs |
| Arena_throughput | 127.99 ns | 7.81M ops/sec | 126.93 ns |
| Evaluator_throughput | 121.22 μs | 8.25K ops/sec | 120.14 μs |
| Parser_throughput | 579.13 ns | 1.73M ops/sec | 576.45 ns |
| Tokenizer_throughput | 319.47 ns | 3.13M ops/sec | 318.05 ns |
| Traditional_throughput | 5.03 μs | 198.77K ops/sec | 4.28 μs |
| Interner_stats | 106.15 ns | 9.42M ops/sec | 105.55 ns |
| With_interning | 2.48 μs | 403.84K ops/sec | 2.46 μs |
| Without_interning | 1.72 μs | 579.88K ops/sec | 1.72 μs |
| Is_keyword_str | 54.09 ns | 18.49M ops/sec | 53.92 ns |
| Keyword_lookup | 2.13 μs | 469.50K ops/sec | 2.12 μs |
| Keyword_stats | 1.52 ns | 659.39M ops/sec | 1.51 ns |
| Memory_estimation | 0.70 ns | 1430.32M ops/sec | 0.69 ns |
| Non_pooled_true | 1.70 ns | 587.77M ops/sec | 1.64 ns |
| Pooled_true | 3.32 ns | 301.18M ops/sec | 3.04 ns |
| Non_pooled_empty_string | 39.19 ns | 25.52M ops/sec | 35.96 ns |
| Pooled_empty_string | 16.77 ns | 59.63M ops/sec | 13.63 ns |
| Non_pooled_one | 3.88 ns | 257.59M ops/sec | 3.65 ns |
| Non_pooled_zero | 2.56 ns | 390.79M ops/sec | 2.32 ns |
| Pooled_one | 4.05 ns | 247.06M ops/sec | 3.71 ns |
| Pooled_zero | 3.19 ns | 313.39M ops/sec | 3.17 ns |
| Batch_variable_set | 330.96 ns | 3.02M ops/sec | 323.68 ns |
| Memory_stats | 13.81 ns | 72.43M ops/sec | 13.75 ns |
| Single_variable_get | 18.53 ns | 53.98M ops/sec | 18.20 ns |
| Single_variable_set | 23.22 ns | 43.07M ops/sec | 22.83 ns |
| Variable_sharing_info | 13.80 ns | 72.46M ops/sec | 13.75 ns |

#### Evaluator

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Complex | 133.13 μs | 7.51K ops/sec | 131.28 μs |
| Complex_arithmetic | 215.29 μs | 4.64K ops/sec | 212.38 μs |
| Complex_chained | 123.39 μs | 8.10K ops/sec | 122.21 μs |
| Complex_filter | 216.02 μs | 4.63K ops/sec | 208.50 μs |
| Complex_nested_func | 214.83 μs | 4.65K ops/sec | 209.78 μs |
| Extremely_complex_multi_lambda | 127.97 μs | 7.81K ops/sec | 126.77 μs |
| Extremely_complex_nested_operations | 218.38 μs | 4.58K ops/sec | 216.56 μs |
| Medium | 141.14 μs | 7.08K ops/sec | 138.38 μs |
| Medium_arithmetic | 216.25 μs | 4.62K ops/sec | 206.29 μs |
| Medium_comparison | 245.25 μs | 4.08K ops/sec | 216.98 μs |
| Medium_function | 135.03 μs | 7.41K ops/sec | 123.89 μs |
| Medium_where | 149.97 μs | 6.67K ops/sec | 132.59 μs |
| Simple | 136.69 μs | 7.32K ops/sec | 134.86 μs |
| Simple_index | 205.15 μs | 4.87K ops/sec | 204.11 μs |
| Simple_literal | 124.20 μs | 8.05K ops/sec | 120.73 μs |
| Simple_nested | 219.27 μs | 4.56K ops/sec | 210.14 μs |
| Simple_property | 144.84 μs | 6.90K ops/sec | 123.42 μs |
| Very_complex_deep_nesting | 215.97 μs | 4.63K ops/sec | 213.71 μs |
| Very_complex_lambda | 223.25 μs | 4.48K ops/sec | 213.29 μs |
| Very_complex_multi_filter | 233.71 μs | 4.28K ops/sec | 214.99 μs |
| Very_complex_nested_logic | 230.29 μs | 4.34K ops/sec | 215.94 μs |
| Very_simple_boolean | 208.82 μs | 4.79K ops/sec | 202.21 μs |
| Very_simple_identifier | 228.12 μs | 4.38K ops/sec | 205.96 μs |
| Very_simple_literal | 207.59 μs | 4.82K ops/sec | 203.82 μs |

#### Fhir_value_allocation

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| 50 | 3.18 μs | 314.48K ops/sec | 3.17 μs |

#### Hex_encoding

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| 100 | 751.42 ns | 1.33M ops/sec | 750.42 ns |
| 1000 | 4.93 μs | 203.04K ops/sec | 4.91 μs |
| 10000 | 59.37 μs | 16.84K ops/sec | 46.24 μs |
| 100000 | 1.13 ms | 882.23 ops/sec | 810.55 μs |
| 100 | 756.58 ns | 1.32M ops/sec | 733.48 ns |
| 1000 | 5.13 μs | 194.76K ops/sec | 5.12 μs |
| 10000 | 49.62 μs | 20.15K ops/sec | 49.51 μs |
| 100000 | 516.54 μs | 1.94K ops/sec | 501.68 μs |

#### Html_encoding

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| 100 | 828.61 ns | 1.21M ops/sec | 787.05 ns |
| 1000 | 6.80 μs | 147.12K ops/sec | 6.28 μs |
| 10000 | 62.90 μs | 15.90K ops/sec | 59.65 μs |
| 50000 | 433.15 μs | 2.31K ops/sec | 420.34 μs |
| 100 | 497.66 ns | 2.01M ops/sec | 448.88 ns |
| 1000 | 3.03 μs | 329.80K ops/sec | 2.98 μs |
| 10000 | 30.56 μs | 32.72K ops/sec | 30.14 μs |
| 50000 | 178.74 μs | 5.59K ops/sec | 174.86 μs |

#### Individual_expressions

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Binary_op | 109.48 ns | 9.13M ops/sec | 108.51 ns |
| Complex_expression | 128.42 ns | 7.79M ops/sec | 127.36 ns |
| Complex_path | 213.06 ns | 4.69M ops/sec | 158.82 ns |
| Function_call | 126.46 ns | 7.91M ops/sec | 124.82 ns |
| Nested_function | 127.65 ns | 7.83M ops/sec | 126.56 ns |
| Simple_identifier | 82.67 ns | 12.10M ops/sec | 82.46 ns |
| Simple_literal | 69.36 ns | 14.42M ops/sec | 69.39 ns |
| Simple_path | 127.81 ns | 7.82M ops/sec | 114.23 ns |
| Binary_op | 124.91 ns | 8.01M ops/sec | 113.16 ns |
| Complex_expression | 1.17 μs | 855.78K ops/sec | 1.15 μs |
| Complex_path | 319.64 ns | 3.13M ops/sec | 305.80 ns |
| Function_call | 413.28 ns | 2.42M ops/sec | 402.04 ns |
| Nested_function | 570.27 ns | 1.75M ops/sec | 572.19 ns |
| Simple_identifier | 58.55 ns | 17.08M ops/sec | 58.14 ns |
| Simple_literal | 32.97 ns | 30.33M ops/sec | 32.81 ns |
| Simple_path | 121.85 ns | 8.21M ops/sec | 121.21 ns |

#### Memory_cloning_baseline

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Medium | 21.01 ms | 47.60 ops/sec | 18.01 ms |
| Small | 2.49 ms | 402.07 ops/sec | 2.36 ms |
| Medium | 101.77 ns | 9.83M ops/sec | 82.28 ns |
| Small | 86.60 ns | 11.55M ops/sec | 83.75 ns |

#### Parallel_cache_throughput

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| 16 | 1.25 ms | 801.68 ops/sec | 1.18 ms |
| 2 | 350.76 μs | 2.85K ops/sec | 337.69 μs |
| 4 | 393.80 μs | 2.54K ops/sec | 389.50 μs |
| 8 | 607.54 μs | 1.65K ops/sec | 604.52 μs |
| 16 | 1.22 ms | 820.85 ops/sec | 1.14 ms |
| 2 | 333.19 μs | 3.00K ops/sec | 313.97 μs |
| 4 | 376.87 μs | 2.65K ops/sec | 371.35 μs |
| 8 | 765.56 μs | 1.31K ops/sec | 632.64 μs |

#### Parser

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Complex | 604.64 ns | 1.65M ops/sec | 579.01 ns |
| Complex_arithmetic | 1.25 μs | 799.57K ops/sec | 1.23 μs |
| Complex_chained | 914.15 ns | 1.09M ops/sec | 857.38 ns |
| Complex_filter | 1.25 μs | 800.79K ops/sec | 1.20 μs |
| Complex_nested_func | 1.65 μs | 607.25K ops/sec | 1.63 μs |
| Extremely_complex_multi_lambda | 3.25 μs | 307.43K ops/sec | 3.09 μs |
| Extremely_complex_nested_operations | 3.78 μs | 264.84K ops/sec | 3.74 μs |
| Medium | 395.90 ns | 2.53M ops/sec | 395.38 ns |
| Medium_arithmetic | 450.08 ns | 2.22M ops/sec | 414.83 ns |
| Medium_comparison | 419.09 ns | 2.39M ops/sec | 418.62 ns |
| Medium_function | 369.56 ns | 2.71M ops/sec | 367.02 ns |
| Medium_where | 666.92 ns | 1.50M ops/sec | 600.13 ns |
| Simple | 121.46 ns | 8.23M ops/sec | 121.05 ns |
| Simple_index | 528.45 ns | 1.89M ops/sec | 491.54 ns |
| Simple_literal | 35.20 ns | 28.41M ops/sec | 33.52 ns |
| Simple_nested | 684.06 ns | 1.46M ops/sec | 541.39 ns |
| Simple_property | 480.48 ns | 2.08M ops/sec | 417.03 ns |
| Very_complex_deep_nesting | 2.87 μs | 348.43K ops/sec | 2.86 μs |
| Very_complex_lambda | 2.24 μs | 445.52K ops/sec | 2.21 μs |
| Very_complex_multi_filter | 2.45 μs | 407.88K ops/sec | 2.44 μs |
| Very_complex_nested_logic | 2.39 μs | 418.00K ops/sec | 2.40 μs |
| Very_simple_boolean | 43.72 ns | 22.87M ops/sec | 43.29 ns |
| Very_simple_identifier | 155.54 ns | 6.43M ops/sec | 149.20 ns |
| Very_simple_literal | 38.86 ns | 25.73M ops/sec | 34.42 ns |

#### Single_threaded_cache

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| 100 | 20.12 μs | 49.70K ops/sec | 20.05 μs |
| 1000 | 197.73 μs | 5.06K ops/sec | 195.09 μs |
| 10000 | 2.11 ms | 474.25 ops/sec | 2.11 ms |
| 100 | 24.01 μs | 41.64K ops/sec | 23.92 μs |
| 1000 | 253.29 μs | 3.95K ops/sec | 251.91 μs |
| 10000 | 3.03 ms | 329.78 ops/sec | 2.98 ms |

#### Target

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Simple_test | 13.75 ns | 72.71M ops/sec | 13.75 ns |

#### Tokenizer

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Complex | 240.51 ns | 4.16M ops/sec | 240.00 ns |
| Complex_arithmetic | 435.37 ns | 2.30M ops/sec | 431.58 ns |
| Complex_chained | 754.63 ns | 1.33M ops/sec | 681.42 ns |
| Complex_filter | 437.74 ns | 2.28M ops/sec | 370.37 ns |
| Complex_nested_func | 764.37 ns | 1.31M ops/sec | 649.87 ns |
| Extremely_complex_multi_lambda | 2.62 μs | 381.23K ops/sec | 2.45 μs |
| Extremely_complex_nested_operations | 1.28 μs | 782.82K ops/sec | 1.20 μs |
| Medium | 178.74 ns | 5.59M ops/sec | 175.12 ns |
| Medium_arithmetic | 158.26 ns | 6.32M ops/sec | 149.64 ns |
| Medium_comparison | 239.32 ns | 4.18M ops/sec | 162.66 ns |
| Medium_function | 389.39 ns | 2.57M ops/sec | 333.19 ns |
| Medium_where | 502.61 ns | 1.99M ops/sec | 456.58 ns |
| Simple | 77.87 ns | 12.84M ops/sec | 76.53 ns |
| Simple_index | 201.42 ns | 4.96M ops/sec | 178.72 ns |
| Simple_literal | 79.08 ns | 12.65M ops/sec | 66.87 ns |
| Simple_nested | 223.79 ns | 4.47M ops/sec | 220.61 ns |
| Simple_property | 238.21 ns | 4.20M ops/sec | 205.30 ns |
| Very_complex_deep_nesting | 989.88 ns | 1.01M ops/sec | 944.60 ns |
| Very_complex_lambda | 1.20 μs | 836.27K ops/sec | 774.91 ns |
| Very_complex_multi_filter | 954.48 ns | 1.05M ops/sec | 884.45 ns |
| Very_complex_nested_logic | 782.33 ns | 1.28M ops/sec | 776.81 ns |
| Very_simple_boolean | 58.20 ns | 17.18M ops/sec | 46.34 ns |
| Very_simple_identifier | 89.17 ns | 11.21M ops/sec | 86.16 ns |
| Very_simple_literal | 36.15 ns | 27.67M ops/sec | 34.95 ns |

#### Tokenizer_streaming

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| 26000 | 401.66 μs | 2.49K ops/sec | 399.79 μs |
| 2600 | 38.73 μs | 25.82K ops/sec | 38.61 μs |
| 260 | 3.97 μs | 251.90K ops/sec | 3.94 μs |
| 26000 | 294.32 μs | 3.40K ops/sec | 293.08 μs |
| 2600 | 27.86 μs | 35.89K ops/sec | 27.74 μs |
| 260 | 2.79 μs | 358.73K ops/sec | 2.77 μs |

#### Uri_encoding

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| 100 | 753.13 ns | 1.33M ops/sec | 740.67 ns |
| 1000 | 10.78 μs | 92.72K ops/sec | 9.87 μs |
| 10000 | 66.13 μs | 15.12K ops/sec | 63.84 μs |
| 50000 | 396.33 μs | 2.52K ops/sec | 369.22 μs |
| 100 | 540.93 ns | 1.85M ops/sec | 485.62 ns |
| 1000 | 3.24 μs | 308.83K ops/sec | 3.15 μs |
| 10000 | 31.42 μs | 31.82K ops/sec | 30.95 μs |
| 50000 | 183.89 μs | 5.44K ops/sec | 175.99 μs |

#### Urlbase64_encoding

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| 100 | 289.96 ns | 3.45M ops/sec | 280.27 ns |
| 1000 | 2.12 μs | 470.85K ops/sec | 1.94 μs |
| 10000 | 20.95 μs | 47.73K ops/sec | 18.21 μs |
| 100000 | 125.15 μs | 7.99K ops/sec | 113.11 μs |
| 100 | 756.10 ns | 1.32M ops/sec | 506.29 ns |
| 1000 | 1.16 μs | 864.02K ops/sec | 1.18 μs |
| 10000 | 6.14 μs | 162.99K ops/sec | 6.10 μs |
| 100000 | 60.03 μs | 16.66K ops/sec | 58.42 μs |

### Performance Summary

**Key Metrics:**
- **Tokenizer**: Processes FHIRPath expressions into tokens
- **Parser**: Builds AST from tokens using Pratt parsing
- **Evaluator**: Executes FHIRPath expressions against data
- **Full Pipeline**: Complete tokenize → parse → evaluate workflow

### Detailed Results

For detailed benchmark results, charts, and statistical analysis, see the HTML reports in `target/criterion/`.

### Running Benchmarks

```bash
# Run core benchmarks
just bench

# Run full benchmark suite
just bench-full

# Update this documentation
just bench-update-docs
```

### Benchmark Infrastructure

- **Framework**: Criterion.rs v0.7
- **Statistical Analysis**: Includes confidence intervals, outlier detection
- **Sample Sizes**: Adaptive sampling for statistical significance
- **Measurement**: Wall-clock time with warm-up cycles

