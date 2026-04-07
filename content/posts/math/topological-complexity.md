---
title: Topological Complexity of Symplectic CW-Complexes
description: We prove that atoroidal symplectic CW-complexes have topological complexity twice their dimension, generalizing a result by Grant and Mescher.
tags: algebraic topology, topological complexity, symplectic geometry
publication: bachelorarbeit
toc: true
---

We prove that every path-connected $2n$-dimensional CW-complex $X$ with $n$ atoroidal
cohomology classes $u_i \in \cH^2_{\text{sing}}(X; F)$ in field coefficients such that $u_1 \cup \dots
\cup u_n \neq 0$ has topological complexity $4n$. This article generalizes a result of Grant
and Mescher [@tc_of_symplectic_manifolds], namely that every connected atoroidal
cohomologically symplectic manifold has topological complexity twice its dimension, and bases
on their work. As this acts as my bachelor thesis, I will introduce the notion of topological
complexity as originally defined by Farber [@tc_of_motion_planning] and give an overview of
[@tc_of_symplectic_manifolds].

# Introduction

Motion planning is a computational problem which is concerned with finding transitions between
states, so-called configurations. This kind of problem frequently occurs in, for example,
engineering and computer science, such as in navigating self-driving cars, moving robot arms or
path-finding algorithms. To formalize, let $X$ be the set of possible, valid and not obstructed
configurations, which in most applications is endowed with a topology. We want to assign to a pair
of initial and final configuration $(A, B) \in X \times X$ a continuous path in $X$ which connects
those two states. If $X$ is path-connected, this is always possible, if not one can still define
this assignment on the path-connected components, but has to decide in advance whether $A$ and $B$
lie in the same one, so assume $X$ to be path-connected. To prevent abrupt motions, a desired
property of such an assignment would be that small changes in initial and final configuration do
not substantially change the assigned transition which leads immediately to the question: Is this
possible?

```definition Path and loop space {#def:path_loop_space}
Let $X$ be a topological space, define the path space $PX \coloneq \cC^0(I, X)$ as the set of
continuous paths in $X$ and the loop space $LX \coloneq \cC^0(S^1, X)$ as the set of all closed
continuous paths in $X$. Together with the compact-open topology, both are topological spaces.
```

```definition Motion planner {#def:motion_planner}
Let $X$ be a path-connected topological space. We have a continuous surjective map
$$\pi \colon PX \twoheadrightarrow X \times X; \; \gamma \mapsto (\gamma(0), \gamma(1))$$
which assigns to a path in $X$ its endpoints. A motion planner is an injective map
$$s \colon X \times X \hookrightarrow PX; \; (A,B) \mapsto \gamma$$
that assigns a path in $X$ to a pair of configurations such that $\pi \circ s =
\id_{X \times X}$, i.e., $s(A,B)(0) = A$ and $s(A,B)(1) = B$.
```

In this context, we can reformulate the problem: Let $X$ be a path-connected topological space.
Does there exist a continuous motion planner of $X$? And indeed, this question can now be answered.

```theorem Continuous motion planner iff contractible {#thm:cts_motion_planner_exists_iff_contractible}
Let $X$ be a path-connected topological space, then there exists a continuous motion planner
for $X$ if, and only if, $X$ is contractible.
```

```proof {#tc-proof-1}
([@tc_of_motion_planning]) Assume that $X$ is contractible, then there exists a point
$p \in X$ and a homotopy $h \colon X \times I \to X$ such that $h\vert_{X \times \{0\}} =
\id_X$ and $h\vert_{X \times \{1\}} = x \mapsto p$. Given $(A,B) \in X \times X$, one gets
a continuous path $t \mapsto h(A, t)$ from $A$ to $p$ and a second continuous path $t
\mapsto h(B, 1-t)$ from $p$ to $B$. By composing those paths, one obtains a continuous
motion planner for X.

Assume now that $X$ has a continuous motion planner $s \colon X
\times X \hookrightarrow PX$. Fix $p \in X$ and define
$$h \colon X \times I \to X; \; (x, t) \mapsto s(x, p)(t).$$
Since now $h\vert_{X \times \{0\}} = \id_X$ and $h\vert_{X \times \{1\}} = x \mapsto p$,
this is a contraction of $X$ to $\{p\}$.
```

This is a disappointing result. In real world applications, the configuration set is rarely
contractible, so for a majority of application relevant configuration spaces there cannot exist a
continuous motion planner. However, one can try to find local continuous motion planners $s_i
\colon U_i \hookrightarrow PX$ which are defined on contractible subsets of $X \times X$. If one
can find a cover of such $U_i$, one can construct a global motion planner with control over the
discontinuities and obtains a measure of how discontinuous the motion planner is, by the minimum
number $n$ of contractible $U_i$ required to cover $X \times X$. The topological complexity of $X$,
denoted $\TC(X)$, is defined to be $n-1$. In this article, we will calculate the topological
complexity of the later defined *symplectic CW-complexes*, a generalization of (c-)symplectic
manifolds, by providing general lower bounds to $\TC$ for CW-complexes with a non-vanishing product
of second degree cohomology classes.

# Topological Complexity

We will now abstract the motion planner problem into a purely mathematical problem, define the
topological complexity and give upper and lower bounds to it, as well as, present some examples.
Across the article, let $\cH^*$ denote an ordinary cohomology theory. Furthermore, we are using the
now common reduced or normalized version of topological complexity, which is one less than Farbers
and yields nicer formulas.

```definition Fibration {#def:fibration}
A fibration is a surjective continuous map between topological spaces $p \colon E
\twoheadrightarrow B$ that satisfies the homotopy lifting property, i.e., for another
topological space $X$, a homotopy $h \colon X \times I \to B$ and a map $f$ that descends to
$h\vert_{X \times \{0\}}$, there exists a homotopy $\tilde h \colon X \times I \to E$ that
lifts $h$ with $f = \tilde h\vert_{X \times \{0\}}$.

Here $p$ is called the fiber projection, $B$ the base space, $E$ the total space and subspaces
of the form $p^{-1}(b)$ for $b \in B$ are called fibers. Injections $s \colon B
\hookrightarrow X$ with the property $p \circ s = \id_B$ are called sections.
```

```definition Schwarz genus {#def:schwarz_genus}
The Schwarz genus of a fibration $p \colon E \to B$, also called sectional category
$\secat(p \colon E \to B)$, is the smallest number $n\in\N$ such that the base space $B$ can
be covered by $n+1$ open subsets $U_i$ such that for each there exists a continuous section
$s_i \colon U_i \hookrightarrow E$. If such a $n$ does not exist, we set
$\secat(p \colon E \to B) = \infty$.
```

In the context of fibrations, we can now define the topological complexity and an important related
notion called Lusternik-Schnirelmann category as special cases of the Schwarz genus.

```lemma End-point evaluation map is a fibration {#lem:end_point_eval_map_is_fibration}
Let $X$ be a path-connected topological space, then $\pi \colon PX \to X \times X$ is a
fibration, called the free path fibration or end-point evaluation map of $X$.
```

```proof {#tc-proof-2}
See for example [@edwin_spanier_algebraic_topology].
```

```definition Lusternik-Schnirelmann category {#def:secat}
The LS-category $\cat(X)$ of a topological space $X$ is the smallest number $n\in\N$ such that
$X$ can be covered with $n + 1$ open contractible subsets. If such an $n$ does not exist, we
set $\cat(X) = \infty$. This is a special case of the sectional category in the sense that
given a fibration $p \colon E \to X$ where $E$ is contractible, we have $\cat(X) =
\secat(p \colon E \to X)$.
```

```definition Topological complexity {#def:tc}
The topological complexity $\TC(X)$ of a path-connected topological space $X$ is defined as
the sectional category of its free path fibration, i.e., $\TC(X) \coloneq
\secat(\pi \colon PX \to X \times X)$. Using
[[#thm:cts_motion_planner_exists_iff_contractible]], this can be understood as the smallest
number $n \in \N$ such that there exists a cover of $X \times X$ by $n+1$ contractible open
subsets $U_i \subset X \times X$.
```

The LS-category and topological complexity are homotopy invariants as shown in [@ls_cat] and
[@tc_of_motion_planning] respectively. Unfortunately, both are rather hard to compute directly,
but one can compute upper bounds using dimension arguments and lower bounds using cohomology.

```definition Covering dimension {#def:dimension}
Let $X$ be a topological space. The covering dimension $\dim(X)$ is the smallest $n \in \N$
such that for each finite open covering $(U_i)_{i \in I}$ there exists a refinement
$(V_j)_{j \in J}$ where each $x \in X$ is contained in at most $n+1$ $V_j$. If such an $n$
does not exist, we set $\dim(X) = \infty$. If $X$ is a CW-complex or manifold, the covering
dimension agrees with the dimension of $X$ as a CW-complex or manifold respectively.
```

The reason for algebraic topology appearing is the following lemma.

```lemma Covering by contractible subsets kills cup products {#lem:covering_of_n_contractible_subsets_implies_product_of_n_classes_is_trivial}
Let $X$ be a topological space. If $X$ admits a covering of $n$ contractible subsets, then any
cup product of $n$ cohomology classes with non-zero degree is trivial.
```

```proof {#tc-proof-3}
Let $R$ be a ring. Let $U_i \subset X$ contractible for $i = 1, \dots, n$ such that $X =
\bigcup_{i=1}^n U_i$. By homotopy invariance we have $\cH^*(X, U_i; R) \cong
\cH^*(X, \{*\}; R)$ and as reduced cohomology only differs from unreduced cohomology in
degrees $k \neq 0$ we have $\cH^k(X, \{*\}; R) \cong \cH^k(X; R)$ for $k \neq 0$. Hence,
the non-relative cup product with $n$ factors for $k_i \neq 0$

$$\cup \colon \bigotimes_{i=1}^n \cH^{k_i}(X; R) \xrightarrow{\cong} \bigotimes_{i=1}^n \cH^{k_i}(X, U_i; R) \xrightarrow{\cup_{\text{rel}}} \cH^{*}(X, \textstyle\bigcup_{i=1}^n U_i; R) = \cH^{*}(X, X; R) = 0$$

is the zero map. Thus, every cup product of $n$ cohomology classes with non-zero degree is
trivial.
```

By contraposition, this means that if there exists a non-trivial cup product of $n$ cohomology
classes, then $X$ does not admit a covering of $n$ contractible subsets. Finding such a non-trivial
cup product will be the important tool to calculate the topological complexity, as it yields a
lower bound.

```definition Cup length and zero-divisor cup length {#def:cl_zcl}
Let $X$ be a topological space and $F$ a field. $\cH^\bullet(X; F) \coloneq
\bigoplus_{i \in \N} \cH^i(X; F)$ is a graded $F$-algebra with the cup product as
multiplication

$$\cup \colon \cH^\bullet(X; F) \otimes \cH^\bullet(X; F) \xrightarrow{\times} \cH^\bullet(X \times X; F) \xrightarrow{\Delta^*} \cH^\bullet(X; F)$$
{#eq:cup_product}

where $\times$ is the cross product isomorphism from the Künneth Theorem and $\Delta \colon X
\to X \times X$ the diagonal inclusion, and $\cH^\bullet(X; F) \otimes \cH^\bullet(X; F)$ is a
graded $F$-algebra with multiplication
$$(\alpha_1 \otimes \beta_1) \cdot (\alpha_2 \otimes \beta_2) = (-1)^{\Abs{\beta_1}\Abs{\alpha_2}} \alpha_1 \alpha_2 \otimes \beta_1 \beta_2$$
where $\Abs{\beta_1}, \Abs{\alpha_2}$ are the cohomology degrees. We call the kernel
$\ker(\cup) \cong \ker(\Delta^*)$ of the $F$-algebra homomorphism [[#eq:cup_product]] the
ideal of zero-divisors. The cup length $\cl(X)$ is the biggest $n \in \N$, such that there
exists a non-trivial cup product of $n$ cohomology classes. The zero-divisor cup length
$\zcl(X)$ is the biggest $n \in \N$, such that there exists a non-trivial cup product of $n$
zero-divisors.
```

Those properties yield bounds for the topological complexity.

```theorem TC bounds {#thm:tc_bounds}
Let $X$ be a path-connected locally contractible paracompact topological space, then

$$\cl(X)  \leq \cat(X) \leq \dim(X)$$
{#eq:cat_bounds_by_cl_dim}

$$\zcl(X) \leq \TC(X) \leq 2\dim(X)$$
{#eq:tc_bounds_by_zcl_dim}

$$\cat(X) \leq \TC(X) \leq \cat(X \times X)$$

$$\cat(X) \leq \TC(X) \leq 2\cat(X)$$
{#eq:tc_bounds_by_cat}
```

```proof {#tc-proof-4}
See [@ls_cat], [@tc_of_motion_planning] and [@topology_of_motion_planning].
```

```theorem Upper TC bound for CW-complexes {#thm:upper_tc_bound_of_cw_by_dim}
Let $X$ be a $(r-1)$-connected CW-complex, then $\TC(X) \leq 2 \dim(X) / r$.
```

```proof {#tc-proof-5}
See [@instabilities_of_robot_motion] or [@schwarz_genus].
Note that this still requires $X$ to be path-connected.
```

This is already enough to calculate the topological complexity of some well understood spaces,
Farber computed it in [@tc_of_motion_planning] for spheres and the compact orientable
two-dimensional surface of genus $g$ $\Sigma_g$. Cohen and Vandembroucq showed in
[@tc_kleinbottle] that the topological complexity of the Klein Bottle is 4, as well as, that
the topological complexity of any non-orientable surface of genus $g \geq 2$ is 4.

$$\TC(S^n) = \case{1, & n \text{ odd} \\ 2, & n \text{ even}} \qquad \TC((S^n)^m) = \case{m, & n \text{ odd} \\ 2m, & n \text{ even}} \qquad \TC(\Sigma_g) = \case{2, & g \leq 1 \\ 4, & g > 1}$$

Another interesting example are projective spaces, as they appear in the motion planning problem of
moving an entire line through euclidean space.

```example Projective spaces {#ex:projective_spaces}
Consider the $n$-dimensional complex projective space $\CP^n$.

It is well known that $\cH_{\text{cell}}^\bullet(\CP^n; \Q) \cong \Q[x]/(x^{n+1})$ with $x$
being a generator of $\cH_{\text{cell}}^2(\CP^n; \Q)$. Due to this ring structure, $x^n$ generates the
non-trivial $\cH_{\text{cell}}^{2n}(\CP^n;\Q)$ and we have $x^{n+1} = 0$, as
$\cH_{\text{cell}}^{2n + 2}(\CP^n; \Q)$ is trivial, so $\cl(\CP^n) = n$ and thus one needs at least
$n + 1$ contractible sets to cover $\CP^n$ by [[#thm:tc_bounds]] [[#eq:cat_bounds_by_cl_dim]].
The standard atlas of $\CP^n$,
$$\cA = \Cu{\kappa_i \colon \CP^n \supset U_i \xrightarrow{\cong} V_i \subset \C^n \simeq \Cu{\text{pt}}} \quad \text{with} \quad U_i = \Cu{[z_0 : \dots : z_n] \in \CP^n \mid z_i \neq 0},$$
is a contractible covering of $n+1$ sets, therefore $\cat(\CP^n) = n$.

To calculate $\TC(\CP^n)$, we associate a class $\bar x \in \cH_{\text{cell}}^\bullet(\CP^n; \Q)
\times \cH_{\text{cell}}^\bullet(\CP^n; \Q)$ to $x$ by letting $\bar x \coloneq 1 \times x -
x \times 1$. Since $\cup(1 \times x - x \times 1) = 1 \cdot x - x \cdot 1 = 0$, this is a
zero-divisor. We have

$$(1 \times x - x \times 1)^{2n} = \sum_{k=0}^{2n} \binom{2n}{k} (1 \times x)^{2n-k} \cup (-x \times 1)^{k} = \sum_{k=0}^{2n} (-1)^k \binom{2n}{k} (x^{2n-k} \times x^k)$$
{#eq:2n_fold_cup_product_of_xbar}

since $x, x \times 1$ and $1 \times x$ have cohomological degree 2 so
$$(1 \times x) \cup (x \times 1) = (x \times 1) \cup (1 \times x) \quad \text{and} \quad (1 \times x) \cup (x \times 1) = (1 \cup x) \times (x \cup 1)$$
thus
$$(1 \times x)^{2n-k} \cup (x \times 1)^{k} = (1 \cup x)^{2n-k} \times (x \cup 1)^{k} = x^{2n-k} \times x^k.$$
As the elements $x^i \times x^j$ generate $\cH_{\text{cell}}^{\bullet}(\CP^n; \Q) \times
\cH_{\text{cell}}^{\bullet}(\CP^n; \Q)$, the summands of [[#eq:2n_fold_cup_product_of_xbar]] do not
cancel each other out, and since $x^n \neq 0$ we get for $k = 0$ the non-trivial summand
$$(-1)^n \binom{2n}{n} (x^n \times x^n) \neq 0,$$
so $\bar x^{2n} \neq 0$, thus $\zcl(\CP^n) \geq 2n$. Therefore, by [[#thm:tc_bounds]]
[[#eq:tc_bounds_by_zcl_dim]] and [[#eq:tc_bounds_by_cat]]
$$2n \leq \zcl(\CP^n) \leq \TC(\CP^n) \leq 2\cl(\CP^n) = 2n,$$
hence $\TC(\CP^n) = 2n$.

For the $n$-dimensional real projective space $\RP^n$ the same argument, using
$\cH_{\text{cell}}^\bullet(\RP^n; \Z_2) \cong \Z_2[x]/(x^{n+1})$ with $x$ being a generator
of $\cH_{\text{cell}}^1(\RP^n; \Z_2)$, shows $\cat(\RP^n) = n$. [[#thm:tc_bounds]]
[[#eq:tc_bounds_by_cat]] now gives $n \leq \TC(\RP^n) \leq 2n$. Unfortunately,
the zero-divisor cup length does not yield a sharp lower bound for all $n$. In fact, finding a
formula for $\TC(\RP^n)$ is an unsolved problem. As shown in
[@motion_planning_in_projective_spaces], for $n \neq 1,3,7$ $\TC(\RP^n)$ is the smallest
number such that there exists an immersion $\RP^n \to \R^{\TC(\RP^n)}$. The topological
complexities of $\RP^n$ for all $n \leq 23$ can be found in that paper too.
```

As seen in the example, to calculate the topological complexity one needs a lower bound that either
meets an upper bound or an explicit construction, which is difficult to achieve for arbitrary
spaces. For this reason, one tries to improve on the bounds or find constraints on the space to
bring them closer together. The topological complexity of a space has only limited nice
interactions with common constructions. When motion planning multiple objects independently for
example, one can model this with product of the configuration spaces.

```theorem TC of products {#thm:tc_of_product}
Let $X,Y$ be path-connected metric spaces, then
$$\TC(X \times Y) \leq \TC(X) + \TC(Y).$$
```

```proof {#tc-proof-6}
See [@tc_of_motion_planning].
```

```theorem TC of wedges {#thm:tc_of_wedge}
Let $X,Y$ be path-connected topological spaces, then
$$\max\{\TC(X), \TC(Y)\} \leq \TC(X \lor Y).$$
If both are polyhedra, then
$$\TC(X \lor Y) \leq \max\{\TC(X), \TC(Y), \cat(X) + \cat(Y)\}.$$
```

```proof {#tc-proof-7}
See [@topology_of_motion_planning].
```

Before moving on, there are various interesting variants of $\TC$ which are worth mentioning. A
weaker variant of $\TC$ was used by Calcines and Vandembroucq in [@ganea_conjecture] to study
the still open $\TC$-Ganea conjecture, which asks for what spaces $X$ the equation
$$\TC(X \times S^k) = \TC(X) + \TC(S^k)$$
holds and originates from the same investigation on the equation
$$\cat(X \times S^k) = \cat(X) + \cat(S^k) = \cat(X) + 1.$$
Parametrized $\TC$ was introduced by Cohen et alii in [@param_tc] and is of use for example
when the configuration space is not fully known. Higher $\TC$ which was defined by Rudyak in
[@higher_tc] and assigns an entire series $\{\TC_n(X)\}_{n=1}^\infty$, which satisfies
$\TC_n(X) \leq \TC_{n+1}(X)$, to a topological space.

# The TC-weight of Cohomology Classes

A successful improvement on the lower bounds is the idea of category weights, here we will focus on
the $\TC$-weight which will replace the zero-divisor cup length lower bound.

```definition TC-weight {#def:tc_weight}
Let $X,Y$ be a path-connected topological spaces, $F$ a field and $k \in \N_0$. The
$\TC$-weight $\wgt(u)$ of a class $u \in \cH^\bullet(X \times X; F)$ is the maximal $k$ such
that $f^*u = 0 \in \cH^\bullet(Y; F)$ for all continuous maps $f \colon Y \to X \times X$ with
$\secat(f^*\pi \colon PX \to Y) \leq k$, where $f^*\pi \colon PX \to Y$ is the pullback
fibration.
```

```theorem TC lower bound by TC-weight {#thm:tc_lower_bound_by_tc_weight}
Let $X$ be a path-connected topological space and $u_0, \dots, u_n \in
\cH^\bullet(X \times X)$, then
$$\wgt(u_0 \cup \dots \cup u_n) \geq \sum_{i = 0}^n \wgt(u_i)$$
and if $u_0 \cup \dots \cup u_n \neq 0$, then
$$\TC(X) \geq \wgt(u_0 \cup \dots \cup u_n).$$
```

```proof {#tc-proof-8}
See [@cohomology_weights].
```

Clearly, we have $\wgt(u) \geq 0$ for all $u \in \cH^\bullet(X \times X; F)$. Since further
$\wgt(u) \geq 1$ if, and only if, $u$ is a zero-divisor ([@cohomology_weights]), the cohomology
weight is a replacement for the zero-divisor cup length lower bound. To improve on it however, we
need cohomology classes with $\wgt(u) \geq 2$. An important construction for finding those is the
(fibrewise) join.

```definition Join and fibrewise join {#def:join}
The join $E_1 *_B E_2$ of two maps $p_1 \colon E_1 \to B$, $p_2 \colon E_2 \to B$ is defined by
the double mapping cylinder of the projections $\pr_1 \colon E_1 \times_B E_2 \to E_1$, $\pr_2
\colon E_1 \times_B E_2 \to E_2$. For the explicit construction, define $E_1 *_B E_2 \coloneq
\Pa{\Pa{E_1 \times_B E_2} \times I \amalg E_1 \amalg E_2} / \sim$ with the relation given by
$(e_1, e_2, 0) \sim e_1$ and $(e_1, e_2, 1) \sim e_2$. The join map $j_{p_1, p_2} \colon
E_1 *_B E_2 \to B$ is given piecewise on the components by $j_{p_1, p_2}([e_1, e_2, t])
\coloneq p_1(e_1) = p_2(e_2)$, $j_{p_1, p_2}([e_1]) \coloneq p_1(e_1)$ and $j_{p_1, p_2}([e_2])
\coloneq p_2(e_2)$. The $n$-fold join of a map $p \colon E \to B$ is defined recursively by
$*_B^0 E \coloneq E$, $*_B^n E \coloneq (*_B^{n-1} E) *_B E$ together with the $n$-fold join
map $j^0p \coloneq p$, $j^np \coloneq j_{j^{n-1}p,p}$. If the joined maps are fibrations, we
will call this a fibrewise join. $E_1 \times_B E_2$ is the pullback of the diagram $E_1
\xrightarrow{p_1} B \xleftarrow{p_2} E_2$ and is explicitly given by $E_1
\times_B E_2 \coloneq \Cu{(x,y) \in E_1 \times E_2 \mid p_1(x) = p_2(y)}$.
```

Specifically, let $\pi_2 \colon P_2X \to X \times X$ be the 2-fold fibrewise join of $\pi \colon PX
\to X \times X$. By definition, $P_2X$ is the double mapping cylinder of the projections $\pr_1,
\pr_2 \colon PX \times_{X \times X} PX \to PX$, where the pullback of $PX
\xrightarrow{\pi} X \times X \xleftarrow{\pi} PX$ is given by pairs of paths with
same endpoints, $PX \times_{X \times X} PX = \Cu{(\alpha, \beta) \in PX \times PX \mid \pi(\alpha)
= \pi(\beta)}$. Define $r_{1,2}: LX \to PX$ by
$$(r_1(\gamma))(t) \coloneq \gamma\Pa{\frac{t}{2}} \quad \text{and} \quad (r_2(\gamma))(t) \coloneq \gamma\Pa{1-\frac{t}{2}} \quad \forall \gamma \in LX, t \in [0,1].$$
$r_1$ takes a loop to its first half and $r_2$ to its second half, however starting at $\gamma(0)$
and reversed in direction, thus $\pi \circ r_1 = \pi \circ r_2$.

Hence we can let $P_2X$ be the double mapping cylinder of $r_1, r_2$, and $\pi_2$ be the whisker
map induced by constant homotopy. Associated to those homotopy pushouts are Mayer-Vietoris sequences in group coefficients $G$, and we call it the *path space sequence*:

$$\cdots \to \cH^k(P_2X; G) \xrightarrow{i_1^* \oplus i_2^*} \cH^k(PX; G) \oplus \cH^k(PX; G) \xrightarrow{r_1^* - r_2^*} \cH^k(LX; G) \xrightarrow{\delta} \cH^{k+1}(P_2X; G) \to \cdots$$

This construction now yields a criterion for a cohomology class $u$
having $\wgt(u) \geq 2$ and a new characterization for being a zero-divisor.

```lemma Join TC-weight condition {#lem:join_tc_weight_geq_2_condition}
Let $X$ be a path-connected topological space, $G$ a group and $u \in
\cH^\bullet(X \times X; G)$. If
$$u \in \ker\Br{\pi_2^\bullet \colon \cH^\bullet(X \times X; G) \to \cH^\bullet(P_2X; G)}$$
then $\wgt(u) \geq 2$.
```

```proof {#tc-proof-9}
See [@tc_of_symplectic_manifolds], this is a direct consequence of a
Lemma of [@schwarz_genus].
```

```lemma Join zero-divisor condition {#lem:join_zero_divisor_condition}
Let $X$ be a path-connected topological space, $G$ be an abelian group and $k \in \N$. A class
$u \in \cH^k(X \times X; G)$ is a zero-divisor if, and only if,
$$\pi_2^*(u) \in \im\Br{\delta: \cH^{k-1}(LX; G) \to \cH^k(P_2X; G)}$$
where $\delta$ denotes the connecting homomorphism of the above constructed path space
sequence.
```

```proof {#tc-proof-10}
([@tc_of_symplectic_manifolds]) Let $\mathrm{ev}_0 \colon PX \to X; \; \gamma
\to \gamma(0)$ be the evaluation map at $0$. As $\mathrm{ev}_0$ is a homotopy equivalence, we have
$$\ker\Br{\Delta^*: \cH^k(X \times X; G) \to \cH^k(X; G)} = \ker\Br{i_j^* \circ \pi_2^*: \cH^k(X \times X; G) \to \cH^k(PX; G)}.$$
Therefore, $u$ is a zero-divisor if, and only if,
$$\pi_2^*(u) \in \ker\Br{i_j^*: \cH^k(P_2X; G) \to \cH^k(PX; G)}$$
for both $j=1,2$ so if, and only if,
$$\pi_2^*(u) \in \ker\Br{i_1^* \oplus i_2^*: \cH^k(P_2X; G) \to \cH^k(PX; G) \oplus \cH^k(PX; G)} = \im\Br{\delta: \cH^{k-1}(LX; G) \to \cH^k(P_2X; G)}$$
where the last step follows from the exactness of the Mayer-Vietoris sequence.
```

```corollary Vanishing boundary implies TC-weight at least 2 {#cor:vanishing_mv_boundary_implies_all_zd_have_tc_weight_geq_2}
Let $X$ be a path-connected topological space, $G$ be an abelian group and $k \in \N$ with
$k \geq 2$. If the connecting homomorphism of the above Mayer-Vietoris vanishes, then every
zero-divisor $u \in \cH^k(X \times X; G)$ satisfies $\wgt(u) \geq 2$.
```

```proof {#tc-proof-11}
([@tc_of_symplectic_manifolds]) By [[#lem:join_zero_divisor_condition]]
and premise we have $\pi_2^*(u) \in \im\;\delta = 0$, hence, by
[[#lem:join_tc_weight_geq_2_condition]], we get $\wgt(u) \geq 2$.
```

# The Connecting Homomorphism of the Path Space Sequence

In this chapter we will look at the construction of the connecting homomorphism of the path space
sequence

$$\cdots \to \cH^k(P_2X; G) \xrightarrow{i_1^* \oplus i_2^*} \cH^k(PX; G) \oplus \cH^k(PX; G) \xrightarrow{r_1^* - r_2^*} \cH^k(LX; G) \xrightarrow{\delta} \cH^{k+1}(P_2X; G) \to \cdots$$

and fix a mistake in [@tc_of_symplectic_manifolds]. The sequence on the level of singular chains is not exact as stated therein, since $i_1^* \oplus i_2^*$ is not injective. By construction of
$P_2X$ as a double mapping cylinder, $P_2X$ contains two copies of $PX$ along with the diagonal
part $LX \times I$ and precisely the chains on this diagonal part $C_{\text{sing}}^*(LX \times I; F)$
cannot be injectively mapped to the chains of the base spaces $C_{\text{sing}}^*(PX; F) \oplus
C_{\text{sing}}^*(PX; F)$. The issue is that the Mayer-Vietoris decomposition was done not on subspaces,
but on spaces homotopy equivalent to those, which of course does not change the resulting Mayer-Vietoris sequence in cohomology, but does make a difference on the level of chains.

By diagram chasing through the correct commutative diagram involving the mapping cylinders $Mr_*$, one can show that the connecting homomorphism $\delta$ can be computed using elements from $C_{\text{sing}}^1(PX; F)$ directly, which is also precisely what [@tc_of_symplectic_manifolds] stated in their proof.

```corollary Connecting homomorphism of the path space sequence {#cor:mv_connecting_homomorphism_of_path_space_sequence}
Let $X$ be a path-connected topological space, $G$ a group, $c \in C_{\text{sing}}^2(X; G)$ closed
and $b_c \in C_{\text{sing}}^1(PX; G)$. If $\del^1(b_c) = \pi^*(\bar c)$, then $\delta([a_c]) =
\pi_2^*([\bar c])$ where $a_c \coloneq r_1^*b_c - r_2^*b_c$.
```

# Topological Complexity of c-symplectic Manifolds

In this chapter we will look at the methods and results of [@tc_of_symplectic_manifolds]. The
spaces of interest are c-symplectic manifolds together with two additional constraints that
increase the lower bounds of $\cat$ and $\TC$. If not stated explicitly, all proofs and
justifications can be found in [@tc_of_symplectic_manifolds].

```definition Cohomologically symplectic manifold {#def:csymplectic_manifold}
A c-symplectic or cohomologically symplectic manifold is a pair $(M, \omega)$ consisting of a
connected closed smooth $2n$-dimensional manifold $M$ and a closed $2$-form $\omega \in
\Omega^2(M)$ such that $[\omega]^n \in \cH_{\text{dR}}^{2n}(M;\R)$ is non-trivial. In particular, a
closed symplectic manifold $(M, \omega)$ is c-symplectic.

*Remark: The original definition in [@tc_of_symplectic_manifolds] does not mention
connectedness or smoothness, however, I am certain that they both assumed and used those
properties.*
```

```definition Aspherical and atoroidal differential forms {#def:aspherical_and_atoroidal_differential_forms}
Let $(M, \omega)$ be a (c-)symplectic manifold. We call $(M, \omega)$ an aspherical
(c-)symplectic manifold if $\int_{S^2} f^* \omega = 0$ for all smooth maps $f \colon S^2 \to M$
and an atoroidal (c-)symplectic manifold if $\int_{T^2} g^* \omega = 0$ for all smooth maps
$g \colon T^2 \to M$. Every c-symplectic atoroidal manifold is c-symplectically aspherical,
since there exists a degree one map $h \colon T^2 \to S^2$ and we have
$$\int_{S^2} f^* \omega = \int_{h(T^2)} f^* \omega = \int_{T^2} (f \circ h)^* \omega = 0.$$
```

The following two statements predate [@tc_of_symplectic_manifolds].

```theorem TC of simply-connected c-symplectic manifolds {#thm:tc_of_simply_connected_csymplectic_manifold}
If $(M, \omega)$ is a simply-connected c-symplectic manifold, then $\cat(M) = \dim(M) / 2$ and
$\TC(M) = \dim(M)$.
```

```theorem TC of aspherical c-symplectic manifolds {#thm:tc_of_aspherical_csymplectic_manifold}
If $(M, \omega)$ is a aspherical c-symplectic manifold, then $\cat(M) = \dim(M)$ but $\TC(M)
\neq 2\dim(M)$.
```

[@tc_of_symplectic_manifolds] now identified a condition to fix the lower $\TC$ bound in
[[#thm:tc_of_aspherical_csymplectic_manifold]], the above defined *atoroidal*. The following
is their main result.

```theorem TC of atoroidal c-symplectic manifolds {#thm:tc_of_atoroidal_csymplectic_manifolds}
If $(M, \omega)$ is an atoroidal c-symplectic manifold, then $\TC(M) = 2\dim(M)$.
```

Their proof uses de Rham cohomology, for that we need to replace $PX$ and $LX$ with smooth
homotopy equivalent spaces. Let $M$ be a finite-dimensional smooth manifold and define the smooth
path space and smooth loop space
$$\cP M \coloneq \cC^\infty(I, M) \quad \text{and} \quad \Lambda M \coloneq \cC^\infty(S^1, M),$$
both equipped with the $\cC^\infty$-Whitney topology are infinite-dimensional Fréchet manifolds
locally modelled on Fréchet spaces $\cC^\infty(I, \R^{\dim M})$ and $\cC^\infty(S^1, \R^{\dim M})$.
Furthermore, there are homotopy equivalences $PM \simeq \cP M, LM \simeq \Lambda M$ and the
endpoint evaluation map $\pi \colon \cP M \to M \times M$ is smooth. The following Lemma makes them
a true replacement.

```lemma TC via smooth path space {#lem:tc_via_smooth_path_space}
If $M$ is a smooth manifold, then $\TC = \secat(\pi \colon \cP M \to M \times M)$.
```

The tangent spaces of $\Lambda M$ and $\cP M$ are given by
$$T_x \cP M \coloneq \Gamma(x^* TM) \quad \forall x \in \cP M \quad \text{and} \quad T_x \Lambda M \coloneq \Gamma(x^* TM) \quad \forall x \in \Lambda M,$$
where $\Gamma$ denotes the space of smooth sections. Tangent vectors at a path or loop are
therefore vector fields along that path or loop. Similar to [[#ex:projective_spaces]], we assign to each $\omega \in
\Omega^k(M)$ the differential form
$$\bar \omega \coloneq 1 \times \omega - \omega \times 1 \coloneq \pr^*_2\omega - \pr^*_1\omega \in \Omega^k(M \times M)$$
using the projections $\pr_i \colon M \times M \to M$ to the $i$-th factor. If $\cd\omega = 0$, then
$\cd\bar\omega = 0$ as well, and $[\bar\omega] \in \cH^k(M \times M; \R)$ is a zero-divisor since
$\pr_1 \circ \Delta = \pr_2 \circ \Delta$. We also need to translate
[[#cor:mv_connecting_homomorphism_of_path_space_sequence]] into de Rham cohomology.

```lemma Connecting homomorphism in de Rham {#lem:mv_connecting_homomorphism_of_path_space_sequence_de_rham}
Let $M$ be a path-connected manifold, $\omega \in \Omega^2(M)$ closed and $\beta_\omega \in
\Omega^1(\cP M)$. If $\cd \beta_\omega = \pi^*(\bar \omega)$, then $\delta([\alpha_\omega]) =
\pi_2^*([\bar \omega])$ where $\alpha_\omega \coloneq r_1^*\beta_\omega - r_2^*\beta_\omega$.
```

```proof {#tc-proof-12}
([@tc_of_symplectic_manifolds]) As the de Rham cohomology groups of
$\cP M$ and $\Lambda M$ are well-defined and the de Rham theorem holds, by naturality of
the cochain maps, this lemma is a direct
consequence of [[#cor:mv_connecting_homomorphism_of_path_space_sequence]].
```

The first step is to define $\beta_\omega$ and verify the condition
$\cd \beta_\omega = \pi^* \bar \omega$.

```definition Beta-omega {#def:beta_omega}
Let $M$ be a path-connected manifold. Given $\omega \in \Omega^2(M)$, define $\beta_\omega \in
\Omega^1(\cP M)$ by
$$(\beta_\omega)_x[\xi] = \int_0^1 \omega_{x(t)}(\dot x(t), \xi(t)) \; \cd t \qquad \forall x \in \cP M, \xi \in T_x\cP M.$$
```

```lemma Beta-omega is constructed properly {#lem:beta_omega_constructed_properly}
If $\omega \in \Omega^2(M)$ is closed, then $\cd\beta_\omega = \pi^*\bar\omega \in
\Omega^2(\cP M)$.
```

```corollary Exact alpha implies TC-weight at least 2 {#cor:exact_alpha_omega_implies_omegabar_tc_weight_geq_2}
Let $\omega \in \Omega^2(M)$ be closed. If $\alpha_\omega$ is exact, then $\wgt([\bar \omega])
\geq 2$.
```

```proof {#tc-proof-13}
$\alpha_\omega$ being exact is equivalent to $[\alpha_\omega] = 0$, hence
$\pi^*_2([\bar\omega]) = 0$ by [[#lem:mv_connecting_homomorphism_of_path_space_sequence_de_rham]].
[[#lem:join_tc_weight_geq_2_condition]] then yields the claim.
```

The second step is to combine [[#cor:exact_alpha_omega_implies_omegabar_tc_weight_geq_2]] with
the non-vanishing closed 2-form $\omega$ of c-symplectic manifolds.

```lemma Pre-theorem {#lem:pre_theorem}
Let $(M, \omega)$ be a c-symplectic manifold. If $[\alpha_\omega] = 0 \in
\cH_{\text{dR}}^1(\Lambda M; \R)$, then $\TC(M) = 2 \dim(M)$.
```

```proof {#tc-proof-14}
([@tc_of_symplectic_manifolds]) Let $2n = \dim M$. By assumption,
$[\omega]^n \in \cH_{\text{dR}}^{2n}(M; \R)$ is non-trivial and therefore $[\bar\omega]^{2n}$ is
non-trivial as well by [[#ex:projective_spaces]]. Thus, by
[[#cor:exact_alpha_omega_implies_omegabar_tc_weight_geq_2]] and
[[#thm:tc_lower_bound_by_tc_weight]],
$$\wgt([\bar\omega]^{2n}) \geq 2n \cdot \wgt([\bar\omega]) \geq 4n = 2\dim(M).$$
With the upper bound $\TC(M) \leq 2\dim(M)$ we get $\TC(M) = 2\dim(M)$.
```

The final step is to prove that the condition of $\omega$ being atoroidal implies exactness of
$\alpha_\omega$.

```proof Proof of [[#thm:tc_of_atoroidal_csymplectic_manifolds]]
([@tc_of_symplectic_manifolds]) To use [[#lem:pre_theorem]] it only remains to show that
$(M, \omega)$ being c-symplectically atoroidal implies $[\alpha_\omega] = 0 \in
\cH_{\text{dR}}^1(\Lambda M; \R)$. For this it suffices to check that integrating $\alpha_\omega$ over
a smooth 1-cycle gives zero, i.e., that
$$\int_{S^1} c^*\alpha_\omega = 0 \qquad \forall c \in \cC^\infty(S^1, \Lambda M).$$
Given such $c$, define $\tilde c \colon T^2 \to M$ by $\tilde c(s,t) \coloneq (c(s))(t)$ for
all $s,t \in S^1$. $\tilde c$ is smooth by [@global_analysis]. Using Fubini's theorem and
the premise of $(M, \omega)$ being atoroidal we get
$$\int_{S^1} c^* \alpha_\omega = \int_0^1 (\alpha_\omega)_{c(s)}[\dot c(s)] \; \cd s = \int_0^1 \int_0^1 \omega_{\tilde c(s,t)}(\del_t \tilde c(s,t), \del_s \tilde c(s,t)) \; \cd t\cd s = \int_{T^2} \tilde c^* \omega = 0.$$
Since $c$ was arbitrary, $[\alpha_\omega] = 0$ and [[#lem:pre_theorem]] yields the claim.
```

# Topological Complexity of symplectic CW-Complexes

As seen, the proof of [[#thm:tc_of_atoroidal_csymplectic_manifolds]] relies on the explicit construction of
$\alpha_\omega$ and requires several calculations. The goal of this article is to provide a nicer and
more general proof using singular cohomology instead. We will start by suitably replacing aspherical
and atoroidal c-symplectic manifolds.

```definition Aspherical and atoroidal cohomology classes {#def:aspherical_and_atoroidal_cohomology_classes}
Let $X$ be a topological space and $G$ a group. A cohomology class $u \in \cH^\bullet(X; G)$ is
called aspherical, if for all continuous maps $f \colon S^2 \to X$, $\cH^\bullet(f)(u) = 0 \in
\cH^\bullet(S^2; G)$. Similarly, $u$ is called atoroidal, if for all continuous maps $g \colon
T^2 \to X$, $\cH^\bullet(g)(u) = 0 \in \cH^\bullet(T^2; G)$. If $u$ is atoroidal, $u$ is also
aspherical, since we have a degree one map $h: T^2 \to S^2$ and thus pulling $u$ back along
$T^2 \xrightarrow{h} S^2 \xrightarrow{f} X$ yields $\cH^\bullet(h)\big(\cH^\bullet(f)(u)\big)
= 0 \implies \cH^\bullet(f)(u) = 0$, since $\cH^\bullet(h)(x) = 0 \iff x = 0$.
```

```definition k-weak symplectic CW-complex {#def:k_weak_symplectic_cw}
Let $F$ be a field. A path-connected $2n$-dimensional CW-complex $X$ together with $k \leq n$
cohomology classes $u_i \in \cH_{\text{sing}}^2(X; F)$ is called $k$-weak symplectic if $u_1 \cup
\dots \cup u_k \neq 0$.
```

```definition Symplectic CW-complex {#def:symplectic_cw}
Let $F$ be a field. Based on [@tc_of_symplectic_manifolds], we will call a path-connected
$2n$-dimensional CW-complex $X$ together with $n$ cohomology classes $u_i \in
\cH_{\text{sing}}^2(X; F)$ symplectic if $u_1 \cup \dots \cup u_n \neq 0$. $(X, u_i)$ is called an
aspherical or atoroidal symplectic CW-complex, if all $u_i$ are aspherical or atoroidal
respectively. Every symplectic CW-complex is a $(\dim X / 2)$-weak symplectic CW-complex.
```

This definition is dependent on the coefficients $F$ as, for example, $\cH^2(\RP^2; \Z_2) \cong
\Z_2$ but $\cH^2(\RP^2; \Q) = 0$, hence we cannot expect spaces to be symplectically aspherical or
atoroidal in all coefficients, if they are in some. However, we only need the existence of such
classes in some arbitrary but fixed field coefficients, as this is sufficient for our purposes.

```lemma Transport of aspherical/atoroidal classes {#lem:transport_of_aspherical_atoroidal_classes}
Let $X$ be a topological space, $R$ a principal ideal domain, $\psi \colon M \to N$ a
$R$-module homomorphism between projective $R$-modules and $u \in \cH^\bullet(X; M)$ aspherical
or atoroidal, then $\psi_*(u) \in \cH^\bullet(X; N)$ is also aspherical or atoroidal
respectively.
```

```proof {#tc-proof-15}
By the Universal Coefficient Theorem we have $\cH^*(X; M) \cong
\Hom_M(\cH_*(X; R), M)$, as $M,N$ are projective. Let $f \colon S^2 \to X$ be
continuous, then by naturality of the $\Hom$-functor,
$$\cH^*(f; N)(\psi_*(u)) = (\cH^*(f; N) \circ \psi_*)(u) = (\psi_* \circ \cH^*(f; M))(u) = \psi_*(0) = 0$$
so $\psi_*(u)$ is aspherical since $f$ was arbitrary. The same argument shows that
$\psi_*(u)$ is atoroidal if $u$ is atoroidal.
```

Symplectic CW-complexes are indeed a generalization of c-symplectic manifolds.

```lemma c-symplectic manifolds are symplectic CW-complexes {#lem:csymplectic_manifolds_are_symplectic_cw}
Each aspherical or atoroidal c-symplectic manifold is an aspherical or atoroidal symplectic
CW-complex respectively.
```

```proof {#tc-proof-16}
Let $(M, \omega)$ be a c-symplectic manifold of dimension $2n$. Since $M$ is smooth and
compact, it admits a finite triangulation, hence has a CW-structure. Using the natural de
Rham isomorphism,
$$\rho(M)^k \colon \cH_{\text{dR}}^k(M; \R) \xrightarrow{A(M)^k} \cH_{\text{sing}, \cC^\infty}^k(M; \R) \xrightarrow{\iota(M)^*} \cH_{\text{sing}}^k(M; \R)$$
with
$$A(M)^k([\omega]) \coloneq \Br{(\sigma \colon \Delta_k \to M) \mapsto \int_{\Delta_k} \sigma^*\omega} \quad \text{and} \quad \iota(M) \colon C_{\text{sing}, \cC^\infty}^k(M; \R) \hookrightarrow C_{\text{sing}}^k(M; \R)$$
define $u_i \coloneq \rho(M)^2([\omega]) \in \cH_{\text{sing}}^2(M; \R)$ for $i = 1, \dots, n$.
Since
$$u_1 \cup \dots \cup u_n = \rho(M)^2([\omega]) \cup \dots \cup \rho(M)^2([\omega]) = \rho(M)^{2n}([\omega]^n) \neq 0,$$
$(M, u_i)$ is a symplectic CW-complex. It remains to show that the properties aspherical
and atoroidal are preserved. Assume $(M, \omega)$ is aspherical and let $f \colon S^2 \to
M$ be continuous. By [@diff_forms_in_at], $f$ is homotopic to a smooth map $g \colon
S^2 \to M$. By naturality of $\rho$, for a smooth singular 2-chain $\sigma \colon \Delta_2 \to S^2$, we get
$$(A(S^2)^2 \circ \cH_{\text{dR}}^2(g))([\omega])(\sigma) = A(S^2)^2([g^*\omega])(\sigma) = \int_{\Delta_2} \sigma^*g^*\omega = 0,$$
since $\omega$ is aspherical. Since $\iota(S^2)^*$ is a group isomorphism, we get
$$\cH_{\text{sing}}^2(f)(u_i) = (\cH_{\text{sing}}^2(f) \circ \rho(M)^2)([\omega]) = (\iota(S^2)^* \circ A(S^2)^2 \circ \cH_{\text{dR}}^2(g))([\omega]) = 0.$$
Thus $u_i$ is aspherical and $(M, u_i)$ an aspherical symplectic CW-complex. The same argument shows the atoroidal case.
```

The following Theorem generalizes [[#thm:tc_of_atoroidal_csymplectic_manifolds]] and is the main result
of this article.

```theorem TC of atoroidal symplectic CW-complexes {#thm:tc_of_atoroidal_symplectic_cw}
Let $(X, u_i)$ be an atoroidal symplectic CW-complex, then $\TC(X) = 2\dim(X)$.
```

To prove this, we start by looking for a non-vanishing cup product of zero-divisors.

```lemma Non-vanishing cup product for weak symplectic CW-complexes {#lem:non_vanishing_cup_product_for_weak_sympl_cw}
Let $(X, u_i, k)$ be a $k$-weak symplectic CW-complex, then $\bar u_1^2 \cup \dots \cup \bar
u_k^2 \neq 0$ and $2k \leq \TC(X)$, where $\bar u_i$ is given as before by $\bar u_i\coloneq 1
\times u_i - u_i \times 1$.
```

```proof {#tc-proof-17}
Using the binomial formula and distributivity, we get
$$\bigcup_{i = 1}^k \bar u_i^2 = \sum_{v \in \{0,1,2\}^k} \Br{\Pa{\prod_{i = 1}^k (-1)^{v_i} \binom{2}{v_i}} \bigcup_{i = 1}^k (u_i^{2-v_i} \times u_i^{v_i})}.$$
In the case of $v = (1, \dots, 1)$, we have a non-trivial summand of the form
$$\prod_{i = 1}^k (-1) \binom{2}{1} \bigcup_{i = 1}^k (u_i \times u_i) = (-2)^k \Br{\Pa{u_1 \cup \dots \cup u_k} \times \Pa{u_1 \cup \dots \cup u_k}} \neq 0$$
as $(X, u_i, k)$ is $k$-weak symplectic and
$$(u_1 \times u_1) \cup (u_2 \times u_2) \cup \dots = \Pa{u_1 \cup \dots \cup u_k} \times \Pa{u_1 \cup \dots \cup u_k}.$$
Therefore, $\bar u_1^2 \cup \dots \cup \bar u_k^2 \neq 0$. [[#thm:tc_bounds]]
[[#eq:tc_bounds_by_zcl_dim]] gives $2k \leq \TC(X)$.
```

This lower bound is already sufficient in the case of 1-connected symplectic CW-complexes.

```theorem TC of simply-connected symplectic CW-complexes {#thm:tc_of_simply_connected_symplectic_cw}
Let $(X, u_i)$ be a 1-connected symplectic CW-complex, then $\TC(X) = \dim(X)$.
```

```proof {#tc-proof-18}
By [[#lem:non_vanishing_cup_product_for_weak_sympl_cw]] we have $\dim(X) \leq \TC(X)$ which
together with $\TC(X) \leq \dim(X)$ from [[#thm:upper_tc_bound_of_cw_by_dim]] gives $\TC(X) =
\dim(X)$.
```

As seen in the definition of aspherical and atoroidal classes, every atoroidal cohomology class
is also aspherical. Furthermore, aspherical and atoroidal symplectic CW-complexes are not simply-connected.

```lemma Atoroidal class implies non-simply-connected {#lem:atoroidal_class_implies_non_simply_connected}
Let $X$ be a topological space and $F$ a field. If $X$ admits a non-trivial aspherical
cohomology class $u \in \cH^2(X; F)$, then $X$ is not simply-connected.
```

```proof {#tc-proof-19}
Assume $X$ to be simply-connected, then the Hurewicz map
$$\Phi \colon \pi_2(X, x_0) \to \cH_2(X); \; [f: S^2 \to X] \mapsto f([S^2])$$
is an isomorphism. Consider the composition of isomorphisms
$$F \colon \cH^2(X; F) \xrightarrow{\text{UCT}} \Hom_F(\cH_2(X), F) \xrightarrow{\Phi^*} \Hom_F(\pi_2(X, x_0), F)$$
given by $F(x) = [f: S^2 \to X] \mapsto f^*x([S^2])$.
Since $u$ is aspherical, $f^* u = 0$ for all $f: S^2 \to X$ and so $F(u) = 0$ which
contradicts the premise of $u \neq 0$. Therefore, $X$ cannot be simply-connected.
```

Since $\pi_1(\CP^n, x_0) = 0$, $\pi_1(\RP^1, x_0) = \Z$ and for $n \geq 0$ $\pi_1(\RP^n, x_0) =
\Zmod{2}$, by [[#thm:tc_of_simply_connected_symplectic_cw]], we have $\TC(\CP^n) = 2n$ and
$\TC(\RP^n) \neq n$, precisely as calculated in [[#ex:projective_spaces]]. For the non-simply
connected case we want to make use of $\TC$-weights, hence we need to construct a $b_c$ as given in
[[#cor:mv_connecting_homomorphism_of_path_space_sequence]]. For this we need currying.

```theorem Currying {#thm:currying}
Let $X,Y,Z \in \Top$ be topological spaces, then we have a natural bijection
$$\Phi \colon \Hom_\Top(X \times Y, Z) \to \Hom_\Top(X, \Hom_\Top(Y, Z))$$
given by $\Phi f(x)(y) \coloneq f(x, y)$ with natural inverse
$$\Psi \colon \Hom_\Top(X, \Hom_\Top(Y, Z)) \to \Hom_\Top(X \times Y, Z)$$
given by $\Psi f(x,y) = f(x)(y)$. If $Y$ is locally compact Hausdorff then both $\Phi$ and
$\Psi$ are homeomorphisms, given that the function spaces are equipped with the compact-open
topology.
```

```proof {#tc-proof-20}
See for example [@category_theory].
```

With this we can replicate [[#def:beta_omega]] and [[#lem:beta_omega_constructed_properly]].

```definition $b_c$ {#def:b_c}
Let $F$ be a field and $X$ be a path-connected CW-complex. Given a $c \in C_{\text{sing}}^2(X; F)$,
define $b_c \in C_{\text{sing}}^1(PX; F)$ by the composition $b_c \colon C^{\text{sing}}_1(PX) \xrightarrow{T} C^{\text{sing}}_2(X) \xrightarrow{c} F$
where $T$ is given by
$$T(\sigma \colon [v_0, v_1] = \Delta_1 \to PX) \coloneq - (\Psi\sigma\lvert_{[v_0, v_1, \hat v_1]} \colon \Delta_2 \to X) + (\Psi\sigma\lvert_{[v_0, \hat v_0, \hat v_1]} \colon \Delta_2 \to X).$$

By uncurrying $\sigma \colon [v_0, v_1] = \Delta_1 \to PX$, we get a map $\Psi\sigma \colon
\Delta_1 \times I \to X$. We can decompose the rectangular domain $\Delta_1 \times I$ into a
lower 2-simplex $[v_0, v_1, \hat v_0]$ and an upper 2-simplex $[v_0, \hat v_0, \hat v_1]$.
Restricting $\Psi\sigma$ to those two simplices yields two singular 2-simplices.
```

```lemma $b_c$ satisfies the boundary condition {#lem:b_c}
Let $F$ be a field and $X$ be a path-connected CW-complex. Given a closed $c \in
C_{\text{sing}}^2(X; F)$, then $\del^1 b_c = \pi^*(\bar c) \in C_{\text{sing}}^2(PX; F)$.
```

```proof {#tc-proof-21}
Let $\sigma \colon [v_0, v_1, v_2] = \Delta_2 \to PX$. We can decompose the domain of
$\Psi\sigma: \Delta_2 \times I \to X$ into the 3-simplices $[v_0, v_1, v_2, \hat v_2], [v_0, v_1, \hat v_1, \hat v_2]$ and $[v_0,
\hat v_0, \hat v_1, \hat v_2]$, and calculate the boundary of $\fC :=
\Psi\sigma\lvert_{[v_0, v_1, v_2, \hat v_2]} - \Psi\sigma\lvert_{[v_0, v_1, \hat v_1, \hat v_2]}
+ \Psi\sigma\lvert_{[v_0, \hat v_0, \hat v_1, \hat v_2]}$ to be

$$\del_1 \fC = \underbrace{\Psi\sigma\lvert_{[\hat v_0, \hat v_1, \hat v_2]} - \Psi\sigma\lvert_{[v_0, v_1, v_2]}}_{\text{top and bottom}} + \underbrace{\Psi\sigma\lvert_{[v_0, v_1, \hat v_1]} - \Psi\sigma\lvert_{[v_0, \hat v_0, \hat v_1]}}_{\text{left face}} + \underbrace{\Psi\sigma\lvert_{[v_1, v_2, \hat v_2]} - \Psi\sigma\lvert_{[v_1, \hat v_1, \hat v_2]}}_{\text{right face}} + \underbrace{\Psi\sigma\lvert_{[v_0, \hat v_0, \hat v_2]} - \Psi\sigma\lvert_{[v_0, v_2, \hat v_2]}}_{\text{back face}}.$$

As $c$ is closed, we have $c(\del_1\fC) = \del^1 c(\fC) = 0$. Note that
$(\pr_1 \circ \pi \circ \sigma)(x,y,z) = \Psi\sigma(x,y,z,0)$ and
$(\pr_2 \circ \pi \circ \sigma)(x,y,z) = \Psi\sigma(x,y,z,1)$,
so $\Psi\sigma\lvert_{[\hat v_0, \hat v_1, \hat v_2]} - \Psi\sigma\lvert_{[v_0, v_1, v_2]}
= \pr_2 \circ \pi \circ \sigma - \pr_1 \circ \pi \circ \sigma$. With this we compute

$$\del^1 b_c(\sigma) = c\Pa{T\Pa{\sigma\lvert_{[v_1, v_2]} - \sigma\lvert_{[v_0, v_2]} + \sigma\lvert_{[v_0, v_1]}}} = c \Pa{\Psi\sigma\lvert_{[\hat v_0, \hat v_1, \hat v_2]} - \Psi\sigma\lvert_{[v_0, v_1, v_2]}} = \Pa{\pi^*(\bar c)}(\sigma).$$

Hence, $\del^1 b_c = \pi^*(\bar c)$.
```

```corollary Exact $a_c$ implies TC-weight at least 2 {#cor:exact_a_c_implies_cbar_tc_weight_geq_2}
Let $F$ be a field, $X$ be a path-connected CW-complex and $c \in C_{\text{sing}}^2(X; F)$ be closed.
If $a_c$, as given in [[#cor:mv_connecting_homomorphism_of_path_space_sequence]], is exact,
then $\wgt([\bar c]) \geq 2$.
```

```proof {#tc-proof-22}
$a_c$ being exact is equivalent to $[a_c] = 0$, hence $\pi^*_2([\bar c]) = 0$ by
[[#cor:mv_connecting_homomorphism_of_path_space_sequence]].
[[#lem:join_tc_weight_geq_2_condition]] then yields the claim.
```

We can use those constructions on the level of chains in cohomology by choosing representatives
of the symplectic classes.

```lemma Atoroidal implies TC-weight at least 2 {#lem:u_atoroidal_implies_baru_tc_weight_geq_2}
Let $F$ be a field, $X$ be a path-connected CW-complex and $u \in \cH_{\text{sing}}^2(X;F)$. If $u$
is atoroidal, then $\wgt(\bar u) \geq 2$.
```

```proof {#tc-proof-23}
Let $c \in C_{\text{sing}}^2(X; F)$ be closed such that $[c] = u$. To use
[[#cor:exact_a_c_implies_cbar_tc_weight_geq_2]], we need to show $[a_c] = 0$. By the
Hurewicz Theorem, we know that $\cH^{\text{sing}}_1(LX; F) \cong \pi_1(LX, x_0)^{\text{ab}}$, hence, if
all pairings of the 1-cocycle $[a_c]$ with 1-cycles $[\gamma: S^1 \to LX]$ vanish, then
$[a_c]$ vanishes as well. With $s,t \in S^1$ we have
$$a_c(\gamma(s)(t)) = (T^*(r_1 - r_2)^*(\Psi\gamma)^*c)(s, t).$$
Note that $\Psi\gamma: T^2 \to X$ is a map from the torus into the space $X$, hence
$$[(\Psi\gamma)^*c] = (\Psi\gamma)^*[c] = (\Psi\gamma)^*u = 0$$
by the premise of $u$ being atoroidal. So
$$[a_c(\gamma)] = T^*(r_1 - r_2)^*(\Psi\gamma)^*[c] = 0.$$
Thus [[#cor:exact_a_c_implies_cbar_tc_weight_geq_2]] yields $\wgt(\bar u) \geq 2$.
```

With this lemma, we now have all pre-requisites to use [[#thm:tc_lower_bound_by_tc_weight]] to
improve upon the lower bound given in [[#lem:non_vanishing_cup_product_for_weak_sympl_cw]].

```lemma k-weak TC lower bound {#lem:k_weak_tc_lower_bound}
Let $(X, u_i, k)$ be a $k$-weak symplectic CW-complex and $0 \leq l \leq k$ the amount of
atoroidal $u_i$, then $2(k - l) + 4l \leq \TC(X)$.
```

```proof {#tc-proof-24}
Applying [[#thm:tc_lower_bound_by_tc_weight]] to the non-trivial product of zero-divisors
given in [[#lem:non_vanishing_cup_product_for_weak_sympl_cw]] along with the fact that
$u_i$ being atoroidal implies $\wgt(\bar u_i) \geq 2$, by
[[#lem:u_atoroidal_implies_baru_tc_weight_geq_2]], gives

$$\TC(X) \geq \wgt\Pa{\bar u_1^2 \cup \dots \cup \bar u_k^2} \geq \sum_{i = 1}^k 2 \wgt(\bar u_i) = \sum_{\substack{0 \leq i \leq k \\ u_i \text{ not ator.}}} 2 \wgt(\bar u_i) + \sum_{\substack{0 \leq i \leq k \\ u_i \text{ ator.}}} 2 \wgt(\bar u_i) \geq 2(k - l) + 4l.$$
```

This lemma now directly proves our main theorem.

```proof Proof of [[#thm:tc_of_atoroidal_symplectic_cw]]
Let $(X, u_i)$ be a atoroidal symplectic CW-complex. By definition, this is a
$(\dim X / 2)$-weak CW-complex with all of those cohomology classes being atoroidal, hence
[[#lem:k_weak_tc_lower_bound]] yields $2 \dim X \leq \TC(X)$ which together with $\TC(X)
\leq 2 \dim X$, by [[#thm:upper_tc_bound_of_cw_by_dim]], yields $\TC(X) = 2 \dim X$.
```

```corollary TC of simply-connected c-symplectic manifolds (via CW) {#cor:tc_simply_connected_csymplectic_via_cw}
If $(M, \omega)$ is a simply-connected c-symplectic manifold, then $\TC(M) = \dim(M)$.
```

```proof {#tc-proof-25}
This now is an immediate consequence of [[#thm:tc_of_simply_connected_symplectic_cw]] and
[[#lem:csymplectic_manifolds_are_symplectic_cw]].
```

```corollary TC of atoroidal c-symplectic manifolds (via CW) {#cor:tc_atoroidal_csymplectic_via_cw}
If $(M, \omega)$ is an atoroidal c-symplectic manifold, then $\TC(M) = 2\dim(M)$.
```

```proof {#tc-proof-26}
This now is an immediate consequence of [[#thm:tc_of_atoroidal_symplectic_cw]] and [[#lem:csymplectic_manifolds_are_symplectic_cw]].
```
