---
title: Vector Spaces
description: Vector spaces, linear maps, and dimension.
series: Abstract Algebra
series_order: 4
tags: math, algebra, linear-algebra
publication: math
toc: true
sources: https://en.wikipedia.org/wiki/Vector_space
---

Linear algebra is the study of vector spaces and linear maps. This presentation follows Roman[^roman] and Axler[^axler], with the algebraic perspective from Lang[^lang].

[^roman]: Steven Roman, *Advanced Linear Algebra*, 3rd ed., Springer, 2008.
[^axler]: Sheldon Axler, *Linear Algebra Done Right*, 3rd ed., Springer, 2015.
[^lang]: Serge Lang, *Algebra*, 3rd ed., Springer, 2002.

## Vector Spaces

```definition Vector Space {#def:vector-space}
A **vector space** over a [[def:field]] $F$ is an [[def:abelian-group]] $(V, +)$ equipped with a scalar multiplication $F \times V \to V$ satisfying:

1. $a(bv) = (ab)v$ for all $a, b \in F$, $v \in V$
2. $1v = v$ for all $v \in V$
3. $a(u + v) = au + av$ for all $a \in F$, $u, v \in V$
4. $(a + b)v = av + bv$ for all $a, b \in F$, $v \in V$
```

```remark {#rem:module}
A vector space is a *module* over a field. Modules over general [[def:ring|rings]] share many properties but can behave very differently — for example, modules over $\mathbb{Z}$ are precisely [[def:abelian-group|abelian groups]], and they need not have a basis.
```

```axiom Vector Space Axioms {#ax:vector-space}
$$\begin{aligned}
u + v &= v + u && \text{(commutativity)} \\
(u + v) + w &= u + (v + w) && \text{(associativity)} \\
0 + v &= v && \text{(zero)} \\
v + (-v) &= 0 && \text{(inverse)} \\
a(bv) &= (ab)v && \text{(scalar associativity)} \\
1v &= v && \text{(scalar identity)} \\
a(u + v) &= au + av && \text{(left distributivity)} \\
(a + b)v &= av + bv && \text{(right distributivity)}
\end{aligned}$$
```

```example {#ex:vector-spaces}
- $F^n$ — the standard $n$-dimensional space over $F$
- $F[x]$ — polynomials over $F$ (infinite-dimensional)
- $M_{m \times n}(F)$ — $m \times n$ matrices over $F$, with $\dim = mn$
- Any [[def:field-extension]] $L/K$ makes $L$ a vector space over $K$
- The solution space of a homogeneous linear ODE $y'' + py' + qy = 0$
```

```definition Subspace {#def:subspace}
A **subspace** $W$ of a [[def:vector-space]] $V$ is a nonempty subset $W \subseteq V$ that is closed under addition and scalar multiplication. Equivalently, $W$ is itself a vector space under the restricted operations.
```

```definition Linear Independence {#def:linear-independence}
Vectors $v_1, \ldots, v_n \in V$ are **linearly independent** if the only solution to $a_1 v_1 + \cdots + a_n v_n = 0$ is $a_1 = \cdots = a_n = 0$.
```

```definition Basis {#def:basis}
A **basis** of a [[def:vector-space]] $V$ is a [[def:linear-independence|linearly independent]] set that spans $V$. Every vector $v \in V$ can be written uniquely as a finite linear combination of basis vectors.
```

```theorem Dimension is Well-Defined {#thm:dimension}
Any two bases of a [[def:vector-space]] $V$ have the same cardinality, called the **dimension** $\dim V$.
```

```proof {#proof:dimension}
We prove the finite-dimensional case. Suppose $\{e_1, \ldots, e_m\}$ and $\{f_1, \ldots, f_n\}$ are both bases with $m < n$. Express each $f_j$ in terms of the $e_i$'s. By the Steinitz exchange lemma, we can replace $e_i$'s with $f_j$'s one at a time while maintaining a basis. After $m$ replacements, $\{f_1, \ldots, f_m\}$ spans $V$ (together with remaining $e$'s), but we still have $f_{m+1}, \ldots, f_n$ to place — contradiction since the $f_j$'s are independent.
```

## Linear Maps

```definition Linear Map {#def:linear-map}
A **linear map** (or **linear transformation**) $T: V \to W$ between [[def:vector-space|vector spaces]] over the same [[def:field]] $F$ is a function satisfying:

$$T(au + bv) = aT(u) + bT(v)$$

for all $a, b \in F$ and $u, v \in V$.
```

```note
Linear maps are the [[def:group-hom|homomorphisms]] of vector spaces. The set $\text{Hom}_F(V, W)$ of all linear maps $V \to W$ is itself a vector space, with $\dim \text{Hom}(V, W) = \dim V \cdot \dim W$.
```

```definition Kernel and Image {#def:linear-kernel-image}
For a [[def:linear-map]] $T: V \to W$:
- The **kernel** is $\ker(T) = \{v \in V : T(v) = 0\}$, a [[def:subspace]] of $V$
- The **image** is $\text{im}(T) = \{T(v) : v \in V\}$, a [[def:subspace]] of $W$
```

```theorem Rank-Nullity Theorem {#thm:rank-nullity}
For a [[def:linear-map]] $T: V \to W$ with $V$ finite-dimensional:

$$\dim V = \dim \ker(T) + \dim \text{im}(T)$$

This is the linear algebra analogue of [[thm:group-iso-1]].
```

```proof {#proof:rank-nullity}
Let $\{u_1, \ldots, u_k\}$ be a basis of $\ker(T)$ and extend to a basis $\{u_1, \ldots, u_k, v_1, \ldots, v_r\}$ of $V$. We claim $\{T(v_1), \ldots, T(v_r)\}$ is a basis of $\text{im}(T)$.

**Spanning**: For any $w = T(v) \in \text{im}(T)$, write $v = \sum a_i u_i + \sum b_j v_j$. Then $T(v) = \sum b_j T(v_j)$.

**Independence**: If $\sum b_j T(v_j) = 0$, then $T(\sum b_j v_j) = 0$, so $\sum b_j v_j \in \ker(T)$. Writing $\sum b_j v_j = \sum a_i u_i$ and using independence of $\{u_1, \ldots, u_k, v_1, \ldots, v_r\}$, all $b_j = 0$.

Thus $\dim \text{im}(T) = r$ and $\dim V = k + r$.
```

```corollary {#cor:injective-surjective}
A [[def:linear-map]] $T: V \to W$ between finite-dimensional spaces of equal dimension is injective if and only if it is surjective.
```

Here is the pattern of isomorphism theorems across algebra:

```tikzcd
\begin{tikzcd}
  V \arrow[r, "T"] \arrow[d, "\pi"'] & W \\
  V/\ker(T) \arrow[ur, "\bar{T}"', dashed, "\cong"] &
\end{tikzcd}
```

Compare with [[thm:group-iso-1]] and [[thm:ring-iso-1]] — the structure is identical.

## Matrices and Change of Basis

Once we fix bases $\mathcal{B} = \{e_1, \ldots, e_n\}$ for $V$ and $\mathcal{C} = \{f_1, \ldots, f_m\}$ for $W$, every [[def:linear-map]] $T: V \to W$ is represented by a matrix $A \in M_{m \times n}(F)$:

$$T(e_j) = \sum_{i=1}^m A_{ij} f_i$$

```definition Change of Basis {#def:change-of-basis}
If $\mathcal{B}$ and $\mathcal{B}'$ are two bases of $V$, the **change of basis matrix** $P$ satisfies $[v]_{\mathcal{B}'} = P[v]_{\mathcal{B}}$, where $[v]_{\mathcal{B}}$ denotes the coordinate vector. If $T$ has matrix $A$ with respect to $\mathcal{B}$, then with respect to $\mathcal{B}'$ it has matrix $P^{-1}AP$.
```

```tip
Two matrices $A$ and $B$ are **similar** ($A = P^{-1}BP$) if and only if they represent the same linear map in different bases. Invariants of similarity (determinant, trace, eigenvalues, characteristic polynomial) are really properties of the linear map, not the matrix.
```

## Dual Spaces

```definition Dual Space {#def:dual-space}
The **dual space** $V^*$ of a [[def:vector-space]] $V$ is the space of all [[def:linear-map|linear maps]] $V \to F$:

$$V^* = \text{Hom}_F(V, F)$$

Elements of $V^*$ are called **linear functionals**.
```

```theorem {#thm:dual-dimension}
If $V$ is finite-dimensional, then $\dim V^* = \dim V$, and $V \cong V^*$ (non-canonically). Moreover, $V \cong V^{**}$ canonically via $v \mapsto (\varphi \mapsto \varphi(v))$.
```

```proof {#proof:dual-dimension}
Given a basis $\{e_1, \ldots, e_n\}$ of $V$, define $e_i^* \in V^*$ by $e_i^*(e_j) = \delta_{ij}$. Then $\{e_1^*, \ldots, e_n^*\}$ is a basis of $V^*$ (the **dual basis**), so $\dim V^* = n$.

The canonical map $\Phi: V \to V^{**}$ given by $\Phi(v)(\varphi) = \varphi(v)$ is injective (if $\varphi(v) = 0$ for all $\varphi$, take $\varphi = e_i^*$ to get $v = 0$). Since $\dim V = \dim V^{**}$, it is an isomorphism by [[cor:injective-surjective]].
```

```danger
The isomorphism $V \cong V^*$ depends on a choice of basis — it is **not canonical**. The isomorphism $V \cong V^{**}$ is canonical. This distinction matters enormously in differential geometry and physics, where confusing a space with its dual leads to subtle errors.
```

## Computational Example

Computing the kernel and image of a linear map in Python[^1]:

[^1]: The `sympy` library provides exact rational arithmetic, avoiding floating-point issues that plague numerical linear algebra.

```python
from sympy import Matrix, Rational

# T: R^3 -> R^2 given by T(x,y,z) = (x+2y-z, 3x+6y-3z)
A = Matrix([
    [1, 2, -1],
    [3, 6, -3]
])

# Kernel (null space)
ker = A.nullspace()
print("Kernel basis:", ker)
# [[-2, 1, 0], [1, 0, 1]]

# Image (column space)
im = A.columnspace()
print("Image basis:", im)
# [[1, 3]]

# Verify rank-nullity: dim(R^3) = dim(ker) + dim(im)
assert len(ker) + len(im) == 3  # 2 + 1 = 3
```

By [[thm:rank-nullity]], $\dim \ker(T) + \dim \text{im}(T) = 3$, which checks out: the kernel has dimension $2$ and the image has dimension $1$.
