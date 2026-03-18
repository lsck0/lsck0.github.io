---
title: Group Presentations
description: Generators, relations, and free groups.
series: Abstract Algebra
series_order: 6
tags: math, algebra
publication: math
toc: true
sources: https://en.wikipedia.org/wiki/Presentation_of_a_group, https://en.wikipedia.org/wiki/Free_group
date: 2026-01-10
---

How do you describe a group? If it's finite and small, you can write down its multiplication table. But this is impractical for large or infinite groups. **Presentations** give a compact description: specify generators and the relations they satisfy. This chapter follows Rotman[^rotman] and Magnus et al.[^magnus].

[^rotman]: Joseph J. Rotman, *An Introduction to the Theory of Groups*, 4th ed., Springer, 1995.
[^magnus]: Wilhelm Magnus, Abraham Karrass & Donald Solitar, *Combinatorial Group Theory*, 2nd ed., Dover, 1976.

## Free Groups

```definition Free Group {#def:free-group}
The **free group** $F(S)$ on a set $S$ is the [[def:group]] consisting of all reduced words in the alphabet $S \cup S^{-1}$, with concatenation followed by reduction as the group operation. It satisfies the universal property: every map $S \to G$ extends uniquely to a [[def:group-hom]] $F(S) \to G$.
```

```example {#ex:free-group}
The free group $F(\{a, b\})$ on two generators contains elements like:

$$e, \quad a, \quad b^{-1}, \quad aba^{-1}b, \quad b^3a^{-2}ba, \quad \ldots$$

The word $ab^{-1}ba$ is reduced (no cancellation possible), but $abb^{-1}a = a^2$ reduces.
```

```theorem Universal Property of Free Groups {#thm:free-group-universal}
For any [[def:group]] $G$ and any function $f: S \to G$, there exists a unique [[def:group-hom]] $\varphi: F(S) \to G$ with $\varphi|_S = f$.
```

```tikzcd
\begin{tikzcd}
  S \arrow[r, hook, "\iota"] \arrow[dr, "f"'] & F(S) \arrow[d, dashed, "\exists!\,\varphi"] \\
  & G
\end{tikzcd}
```

```remark {#rem:free-group-rank}
The **rank** of a free group is $|S|$. The Nielsen-Schreier theorem states that every [[def:subgroup]] of a free group is free. If $F$ has rank $n$ and $H \leq F$ has index $m$, then $H$ has rank $1 + m(n-1)$.
```

## Presentations

```definition Group Presentation {#def:presentation}
A **presentation** of a [[def:group]] $G$ is an expression $G = \langle S \mid R \rangle$ where $S$ is a set of **generators** and $R \subseteq F(S)$ is a set of **relations**. This means $G \cong F(S) / N(R)$, where $N(R)$ is the [[def:normal-subgroup|normal closure]] of $R$ in the [[def:free-group]] $F(S)$.
```

```example {#ex:presentations}
| Group | Presentation | Order |
|---|---|---|
| $\mathbb{Z}$ | $\langle a \mid \;\rangle$ | $\infty$ |
| $\mathbb{Z}/n\mathbb{Z}$ | $\langle a \mid a^n \rangle$ | $n$ |
| $D_n$ (dihedral) | $\langle r, s \mid r^n, s^2, srsr \rangle$ | $2n$ |
| $Q_8$ (quaternion) | $\langle i, j \mid i^4, i^2j^{-2}, iji^{-1}j \rangle$ | $8$ |
| $S_3$ | $\langle \sigma, \tau \mid \sigma^3, \tau^2, (\sigma\tau)^2 \rangle$ | $6$ |
```

```definition Finitely Generated and Finitely Presented {#def:finitely-presented}
A [[def:group]] $G$ is **finitely generated** if $G = \langle S \mid R \rangle$ with $|S| < \infty$. It is **finitely presented** if both $|S|$ and $|R|$ are finite.
```

```theorem {#thm:every-group-quotient}
Every [[def:group]] is a quotient of a [[def:free-group]].
```

```proof {#proof:every-group-quotient}
Given a group $G$, take $S = G$ as the generating set. The identity map $\text{id}: G \to G$ extends by [[thm:free-group-universal]] to a surjective homomorphism $\varphi: F(G) \to G$. By [[thm:group-iso-1]], $G \cong F(G)/\ker(\varphi)$.
```

## The Word Problem

```conjecture Decidability by Structure {#conj:word-decidability}
The word problem may be decidable for all **automatic groups** — groups whose elements can be represented by strings accepted by a finite automaton, with multiplication also computable by automata.
```

```remark {#rem:word-problem}
Given a [[def:presentation]] $\langle S \mid R \rangle$, the **word problem** asks: given two words $w_1, w_2$ in the generators, do they represent the same group element? This problem is undecidable in general[^novikov], but decidable for many specific classes:

[^novikov]: Pyotr S. Novikov, "On the algorithmic unsolvability of the word problem in group theory," *Trudy Mat. Inst. Steklova* **44**, 1–143, 1955.
```

| Class | Decidable? | Method |
|---|---|---|
| Finite groups | Yes | Enumerate cosets |
| Abelian groups | Yes | Smith normal form |
| Free groups | Yes | Reduction |
| Hyperbolic groups | Yes | Dehn's algorithm |
| General f.p. groups | **No** | Novikov 1955 |

> The undecidability of the word problem is one of the most striking results in mathematics. It means there can be no algorithm that takes an arbitrary group presentation and two words, and always correctly determines whether they represent the same element.

```tip
In practice, computational group theory packages like GAP and Magma implement efficient algorithms for the decidable cases. The **Todd-Coxeter algorithm** enumerates cosets of a subgroup and can sometimes solve the word problem even when the general problem is undecidable.
```

## Cayley Graphs

```definition Cayley Graph {#def:cayley-graph}
The **Cayley graph** $\Gamma(G, S)$ of a [[def:group]] $G$ with generating set $S$ is the directed graph with vertex set $G$ and edges $g \to gs$ for each $g \in G$, $s \in S$.
```

```example Cayley Graph of $\mathbb{Z}/4\mathbb{Z}$ {#ex:cayley}
The Cayley graph of $\mathbb{Z}/4\mathbb{Z} = \langle a \mid a^4 \rangle$ with generator $\{a\}$ is a directed cycle:
```

```tikz
\begin{tikzpicture}[scale=1.5, every node/.style={circle, draw, minimum size=0.6cm, inner sep=0pt}]
  \node (0) at (0,1) {$0$};
  \node (1) at (1,0) {$1$};
  \node (2) at (0,-1) {$2$};
  \node (3) at (-1,0) {$3$};
  \draw[->, thick] (0) -- (1);
  \draw[->, thick] (1) -- (2);
  \draw[->, thick] (2) -- (3);
  \draw[->, thick] (3) -- (0);
\end{tikzpicture}
```

```example Cayley Graph of $D_3$ {#ex:cayley-d3}
The dihedral group $D_3 = \langle r, s \mid r^3, s^2, srsr \rangle$ with generators $\{r, s\}$. The Cayley graph has $6$ vertices with $r$-edges forming two triangles and $s$-edges connecting them:
```

```tikz
\begin{tikzpicture}[scale=1.8, vertex/.style={circle, draw, minimum size=0.5cm, inner sep=0pt, font=\scriptsize}]
  % Outer triangle (rotations)
  \node[vertex] (e) at (90:1.2) {$e$};
  \node[vertex] (r) at (210:1.2) {$r$};
  \node[vertex] (r2) at (330:1.2) {$r^2$};
  % Inner triangle (reflections)
  \node[vertex] (s) at (90:0.5) {$s$};
  \node[vertex] (rs) at (210:0.5) {$rs$};
  \node[vertex] (r2s) at (330:0.5) {$r^2s$};
  % r-edges (outer)
  \draw[->, blue, thick] (e) to[bend right=15] (r);
  \draw[->, blue, thick] (r) to[bend right=15] (r2);
  \draw[->, blue, thick] (r2) to[bend right=15] (e);
  % r-edges (inner)
  \draw[->, blue, thick] (s) to[bend left=15] (r2s);
  \draw[->, blue, thick] (r2s) to[bend left=15] (rs);
  \draw[->, blue, thick] (rs) to[bend left=15] (s);
  % s-edges
  \draw[<->, red, dashed] (e) -- (s);
  \draw[<->, red, dashed] (r) -- (rs);
  \draw[<->, red, dashed] (r2) -- (r2s);
\end{tikzpicture}
```

Blue arrows are $r$ (rotation), red dashed lines are $s$ (reflection, self-inverse).

```info
Cayley graphs connect group theory to geometric group theory and topology. The **growth rate** of a Cayley graph (how the number of vertices within distance $n$ from $e$ scales) is a quasi-isometry invariant of the group, linking algebra to large-scale geometry[^lyndon].

[^lyndon]: Roger C. Lyndon & Paul E. Schupp, *Combinatorial Group Theory*, Springer, 2001.
```

## Implementing Presentations

Here is a simple implementation of a finitely presented group in Haskell, representing elements as reduced words:

```haskell
-- A word is a list of (generator, exponent) pairs
type Word = [(Char, Int)]

-- Reduce a word by cancelling adjacent inverse generators
reduce :: Word -> Word
reduce [] = []
reduce [x] = [x]
reduce ((g1,e1):(g2,e2):rest)
  | g1 == g2  = let e = e1 + e2
                in if e == 0 then reduce rest
                   else reduce ((g1, e) : rest)
  | otherwise = (g1, e1) : reduce ((g2, e2) : rest)

-- Multiply two words
multiply :: Word -> Word -> Word
multiply w1 w2 = reduce (w1 ++ w2)

-- Inverse of a word
invert :: Word -> Word
invert = reverse . map (\(g, e) -> (g, -e))
```

```danger
This code only handles free groups correctly. For a general presentation $\langle S \mid R \rangle$, determining equality of elements requires solving the word problem, which is undecidable in general!
```

[^1]: The dihedral group $D_n$ is the symmetry group of a regular $n$-gon. It has $2n$ elements: $n$ rotations and $n$ reflections.
