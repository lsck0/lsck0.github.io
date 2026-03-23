---
title: A Visual Guide to Sorting Algorithms
description: Comparing common sorting algorithms with complexity analysis and flow diagrams.
tags: cs, algorithms
publication: cs
toc: true
---

Sorting is one of the most fundamental operations in computer science. Let's compare a few classic algorithms.

## Complexity Overview

| Algorithm   | Best          | Average       | Worst         | Space       |
| ----------- | ------------- | ------------- | ------------- | ----------- |
| Bubble Sort | $O(n)$        | $O(n^2)$      | $O(n^2)$      | $O(1)$      |
| Merge Sort  | $O(n \log n)$ | $O(n \log n)$ | $O(n \log n)$ | $O(n)$      |
| Quick Sort  | $O(n \log n)$ | $O(n \log n)$ | $O(n^2)$      | $O(\log n)$ |

## Merge Sort in Rust

```rust
fn merge_sort(arr: &mut [i32]) {
    let len = arr.len();
    if len <= 1 {
        return;
    }
    let mid = len / 2;
    merge_sort(&mut arr[..mid]);
    merge_sort(&mut arr[mid..]);
    let mut merged = arr.to_vec();
    let (mut i, mut j, mut k) = (0, mid, 0);
    while i < mid && j < len {
        if arr[i] <= arr[j] {
            merged[k] = arr[i];
            i += 1;
        } else {
            merged[k] = arr[j];
            j += 1;
        }
        k += 1;
    }
    merged[k..].copy_from_slice(if i < mid { &arr[i..mid] } else { &arr[j..len] });
    arr.copy_from_slice(&merged);
}
```

## Decision Flow

```mermaid
graph TD
    A[Need to sort?] --> B{Data size}
    B -->|Small n < 20| C[Insertion Sort]
    B -->|Medium| D{Stability needed?}
    B -->|Large| E{Worst-case guarantee?}
    D -->|Yes| F[Merge Sort]
    D -->|No| G[Quick Sort]
    E -->|Yes| F
    E -->|No| G
```

The choice of sorting algorithm depends heavily on context: data size, whether stability matters, memory constraints, and whether the data is nearly sorted.
