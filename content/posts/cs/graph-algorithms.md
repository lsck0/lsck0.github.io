---
title: Graph Algorithms
description: BFS, DFS, Dijkstra, and beyond — traversing and computing on graphs.
tags: cs, algorithms
publication: cs
toc: true
---

Graphs are fundamental data structures. This post covers classic graph algorithms.

## Breadth-First Search (BFS)

BFS explores level by level, finding shortest paths in unweighted graphs:

```python
from collections import deque

def bfs(graph, start):
    visited = {start}
    queue = deque([start])
    while queue:
        node = queue.popleft()
        print(node, end=' ')
        for neighbor in graph[node]:
            if neighbor not in visited:
                visited.add(neighbor)
                queue.append(neighbor)
```

## Depth-First Search (DFS)

DFS goes deep before backtracking. Useful for topological sort and cycle detection:

```python
def dfs(graph, node, visited=None):
    if visited is None:
        visited = set()
    if node in visited:
        return
    visited.add(node)
    print(node, end=' ')
    for neighbor in graph[node]:
        dfs(graph, neighbor, visited)
```

## Dijkstra's algorithm

Finds shortest paths in weighted graphs with nonnegative weights:

```python
import heapq

def dijkstra(graph, start):
    dist = {node: float('inf') for node in graph}
    dist[start] = 0
    pq = [(0, start)]
    while pq:
        d, node = heapq.heappop(pq)
        if d > dist[node]:
            continue
        for neighbor, weight in graph[node]:
            new_dist = d + weight
            if new_dist < dist[neighbor]:
                dist[neighbor] = new_dist
                heapq.heappush(pq, (new_dist, neighbor))
    return dist
```

## Topological sort

Order vertices in a DAG such that all edges point forward:

```python
def topological_sort(graph):
    visited = set()
    stack = []
    def dfs(node):
        if node in visited:
            return
        visited.add(node)
        for neighbor in graph[node]:
            dfs(neighbor)
        stack.append(node)
    for node in graph:
        dfs(node)
    return stack[::-1]
```

## Complexity

| Algorithm | Time | Space |
|-----------|------|-------|
| BFS | O(V + E) | O(V) |
| DFS | O(V + E) | O(V) |
| Dijkstra | O((V + E) log V) | O(V) |
| Topological sort | O(V + E) | O(V) |
