---
title: Rings
description: Rings, ideals, and ring homomorphisms.
series: Abstract Algebra
series_order: 2
tags: math, algebra
publication: math
toc: true
sources: https://en.wikipedia.org/wiki/Ring_(mathematics)
---

This post builds on the group theory developed in [Groups](/blog/math/group-theory). The treatment follows Lang[^lang] and Atiyah & Macdonald[^am].

[^lang]: Serge Lang, _Algebra_, 3rd ed., Springer, 2002.

[^am]: Michael F. Atiyah & Ian G. Macdonald, _Introduction to Commutative Algebra_, Addison-Wesley, 1969.

## Rings

```definition Ring {#def:ring}
A **ring** $(R, +, \cdot)$ is a set $R$ equipped with two binary operations such that:

1. $(R, +)$ is an [[def:abelian-group]]
2. $(R, \cdot)$ is a monoid (associative with identity $1_R$)
3. **Distributivity**: $a(b + c) = ab + ac$ and $(a + b)c = ac + bc$ for all $a, b, c \in R$
```

```definition Commutative Ring {#def:commutative-ring}
A ring $R$ is **commutative** if $ab = ba$ for all $a, b \in R$.
```

```example {#ex:rings}
- $(\mathbb{Z}, +, \cdot)$ — the integers
- $(\mathbb{Z}[x], +, \cdot)$ — polynomials with integer coefficients
- $(M_n(\mathbb{R}), +, \cdot)$ — $n \times n$ matrices (not commutative for $n \geq 2$!)
- $(\mathbb{Z}/n\mathbb{Z}, +, \cdot)$ — integers modulo $n$
```

```warning
Some authors do not require rings to have a multiplicative identity. In this series, **all rings are unital** — they have a $1_R$ with $1_R \cdot a = a \cdot 1_R = a$.
```

```axiom Ring Axioms {#ax:ring-axioms}
$$\begin{aligned}
a + (b + c) &= (a + b) + c && \text{(associativity)} \\
a + b &= b + a && \text{(commutativity)} \\
a + 0 &= a && \text{(identity)} \\
a + (-a) &= 0 && \text{(inverse)} \\
a(bc) &= (ab)c && \text{(associativity)} \\
1 \cdot a &= a \cdot 1 = a && \text{(identity)} \\
a(b + c) &= ab + ac && \text{(left distributivity)} \\
(a + b)c &= ac + bc && \text{(right distributivity)}
\end{aligned}$$
```

The hierarchy of algebraic structures:

| Structure                | Addition      | Multiplication                         |
| ------------------------ | ------------- | -------------------------------------- |
| [[def:abelian-group]]    | abelian group | —                                      |
| [[def:ring]]             | abelian group | monoid + distributivity                |
| [[def:commutative-ring]] | abelian group | commutative monoid + dist.             |
| Integral domain          | abelian group | comm. monoid + no zero divisors        |
| [[def:field]]            | abelian group | abelian group (on $R \setminus \{0\}$) |

```definition Zero Divisor {#def:zero-divisor}
A nonzero element $a$ in a [[def:ring]] $R$ is a **zero divisor** if there exists a nonzero $b \in R$ with $ab = 0$ or $ba = 0$.
```

```definition Integral Domain {#def:integral-domain}
An **integral domain** is a [[def:commutative-ring]] with $1 \neq 0$ and no [[def:zero-divisor|zero divisors]].
```

```example {#ex:zero-divisors}
In $\mathbb{Z}/6\mathbb{Z}$, we have $\bar{2} \cdot \bar{3} = \bar{0}$, so $\bar{2}$ and $\bar{3}$ are zero divisors. But $\mathbb{Z}/5\mathbb{Z}$ is an integral domain (in fact, a field).
```

## Ideals

```definition Ideal {#def:ideal}
An **ideal** $I$ of a [[def:ring]] $R$ is an additive [[def:subgroup]] $I \leq (R, +)$ that absorbs multiplication:

$$r \in R, \; a \in I \implies ra \in I \text{ and } ar \in I$$
```

```remark {#rem:ideal-normal}
Ideals are the ring-theoretic analogue of [[def:normal-subgroup|normal subgroups]]. Just as normal subgroups are exactly the kernels of group homomorphisms, ideals are exactly the kernels of ring homomorphisms.
```

```definition Principal Ideal {#def:principal-ideal}
An [[def:ideal]] $I \subseteq R$ is **principal** if $I = (a) = \{ra : r \in R\}$ for some $a \in R$.
```

```definition Principal Ideal Domain {#def:pid}
A **principal ideal domain** (PID) is an [[def:integral-domain]] in which every [[def:ideal]] is [[def:principal-ideal|principal]].
```

```example {#ex:ideals}
In $\mathbb{Z}$, every ideal is principal: $I = (n) = n\mathbb{Z}$ for some $n \geq 0$. So $\mathbb{Z}$ is a PID. The polynomial ring $\mathbb{Z}[x]$ is *not* a PID — the ideal $(2, x) = \{2f + xg : f, g \in \mathbb{Z}[x]\}$ is not principal.
```

## Ring Homomorphisms

```definition Ring Homomorphism {#def:ring-hom}
A map $\varphi: R \to S$ between [[def:ring|rings]] is a **ring homomorphism** if:

$$\varphi(a + b) = \varphi(a) + \varphi(b), \quad \varphi(a \cdot b) = \varphi(a) \cdot \varphi(b), \quad \varphi(1_R) = 1_S$$
```

```theorem First Isomorphism Theorem for Rings {#thm:ring-iso-1}
If $\varphi: R \to S$ is a [[def:ring-hom]], then $\ker(\varphi)$ is an [[def:ideal]] of $R$ and

$$R / \ker(\varphi) \cong \text{im}(\varphi)$$
```

```proof {#proof:ring-iso-1}
The proof mirrors [[thm:group-iso-1]] for groups. The key additional step is verifying that the kernel is an ideal (not just a normal subgroup) and that the induced map preserves multiplication.
```

```tikzcd
\begin{tikzcd}
  R \arrow[r, "\varphi"] \arrow[d, "\pi"'] & S \\
  R/\ker(\varphi) \arrow[ur, "\bar{\varphi}"', dashed, "\cong" near start] &
\end{tikzcd}
```

## Prime and Maximal Ideals

```definition Prime Ideal {#def:prime-ideal}
An [[def:ideal]] $\mathfrak{p} \subsetneq R$ is **prime** if whenever $ab \in \mathfrak{p}$, then $a \in \mathfrak{p}$ or $b \in \mathfrak{p}$.
```

```definition Maximal Ideal {#def:maximal-ideal}
An [[def:ideal]] $\mathfrak{m} \subsetneq R$ is **maximal** if there is no ideal $I$ with $\mathfrak{m} \subsetneq I \subsetneq R$.
```

The containment is strict:

```tikz
\begin{tikzpicture}
  \node (max) at (0,3) {Maximal ideals};
  \node (prime) at (0,1.5) {Prime ideals};
  \node (ideal) at (0,0) {All ideals};
  \draw[thick, ->] (max) -- (prime) node[midway, right] {$\subseteq$};
  \draw[thick, ->] (prime) -- (ideal) node[midway, right] {$\subseteq$};
  \node[right, text width=5cm, font=\small] at (2, 3) {$R/\mathfrak{m}$ is a field};
  \node[right, text width=5cm, font=\small] at (2, 1.5) {$R/\mathfrak{p}$ is an integral domain};
\end{tikzpicture}
```

```proposition {#prop:maximal-is-prime}
Every [[def:maximal-ideal]] in a [[def:commutative-ring]] is a [[def:prime-ideal]].
```

```proof {#proof:maximal-prime}
Let $\mathfrak{m}$ be maximal and suppose $ab \in \mathfrak{m}$ with $a \notin \mathfrak{m}$. Then $\mathfrak{m} + (a) = R$ by maximality, so $1 = m + ra$ for some $m \in \mathfrak{m}$, $r \in R$. Then $b = mb + rab \in \mathfrak{m}$.
```

```theorem {#thm:quotient-field}
Let $R$ be a [[def:commutative-ring]] and $I$ an [[def:ideal]]. Then:
- $R/I$ is an [[def:integral-domain]] if and only if $I$ is a [[def:prime-ideal]]
- $R/I$ is a [[def:field]] if and only if $I$ is a [[def:maximal-ideal]]
```

```info
The **Zariski topology** on the set of prime ideals $\text{Spec}(R)$ connects commutative algebra with algebraic geometry. This perspective, developed by Grothendieck, revolutionized both fields[^am].
```

## Unique Factorization

```definition Unique Factorization Domain {#def:ufd}
An [[def:integral-domain]] $R$ is a **unique factorization domain** (UFD) if every nonzero non-unit can be written as a product of irreducible elements, uniquely up to order and associates[^1].
```

[^1]: Two elements $a, b$ are **associates** if $a = ub$ for some unit $u$. In $\mathbb{Z}$, the associates of $6$ are $\pm 6$.

```proposition {#prop:pid-ufd}
Every [[def:pid|PID]] is a [[def:ufd|UFD]].
```

The hierarchy of integral domains:

```mermaid
graph LR
    F[Field] --> ED[Euclidean Domain]
    ED --> PID[PID]
    PID --> UFD[UFD]
    UFD --> ID[Integral Domain]
    ID --> CR[Commutative Ring]
```

> Not every UFD is a PID. The polynomial ring $\mathbb{Z}[x]$ is a UFD but not a PID: the ideal $(2, x)$ is not principal.
