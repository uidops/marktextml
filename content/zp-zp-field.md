---
title: ℤₚ × ℤₚ Field
date: 2024-01-10
slug: zp-zp-field
description: Notes on how to define the Zp×Zp field
tags: [math, abstract-algebra]
banner: assets/img/zp-zp-field.svg
---

**Problem**:

$$
\forall p \in \mathbb{P}, \mathbb{Z}_{p} \times \mathbb{Z}_{p}= \{(a,b):a,b \in Z_{p} \}
$$

*Does* $(\mathbb{Z}_{p} \times \mathbb{Z}p,+,.)$ *define a field? If so, how should its addition and multiplication operations be defined?*

Let’s look at $\mathbb{Z}_{p}$.\
$(\mathbb{Z}_{p},+)=(\mathbb{Z}/p \mathbb{Z},+)$ defines a cyclic group under modular addition, and $(\mathbb{Z}_{p}^{*},.)$ is a group under modular multiplication, where $\mathbb{Z}_{p}^{*}= \{a \in \mathbb{Z}_{p}:gcd(a,p)=1 \}$ and $| \mathbb{Z}_{p}^{*}|= \varphi(p)$ (in this case, the integers from 1 to p-1).\
 We also know that $(\mathbb{Z}_{p},+,.)$ is a field, unlike $(\mathbb{Z},+,.)$, which is only an integral domain; this is easy to prove.

Now let’s look at polynomial rings.\
**Theorem 1.1**: *$(R[x],+,.)$, the polynomial ring over the integral domain $R$, defines an integral domain.*

$$
x^{n}=(...,0,0, \underline{1},0,0,...)
$$

$$
R[x]= \{\Sigma_{n=0}^{\infty}a_{n}x^{n}:a_{n} \in R \}
$$

$$
(a_{0}+a_{1}x+...+a_{n}x^{n})+(b_{0}+b_{1}x^{1}+...+b_{m}x^{m})=(a_{0}+b_{0})+(a_{1}+b_{1})x+...
$$

$$
(a_{0}+a_{1}x+...+a_{n}x^{n}).(b_{0}+b_{1}x^{1}+...+b_{m}x^{m})=c_{0}+c_{1}x+...+c_{n+m}x^{n+m}
$$

*where* $c_{k}= \Sigma_{i=0}^{k}a_{i}b_{k-i}$*.*\
 Proof.

- unity: $\forall f(x) \in R[x],(1x^{0}).f(x)=f(x).(1x^{0})=f(x)$
- commutative ring: $c_{k}= \Sigma_{i=0}^{k}a_{i}b_{k-i}= \Sigma_{i=0}^{k}b_{i}a_{k-i}$
- no nonzero zero divisors:\
     This means $f(x).g(x)=0 \longrightarrow f(x)=0 \vee g(x)=0$, since we have $c_{k}= \Sigma_{i=0}^{k}a_{i}b_{k-i}$ and $R$ is an integral domain, $c_{k}$ would not be zero if $f(x)$ and $g(x)$ are both non-null polynomials. $\square$

**Definition (Maximal Ideal)**: *An ideal $I$ is a maximal ideal of $(R,+,.)$ iff there is no ideal $I_{1}$ in $R$ such that $I \subset I_{1} \subset R$.*\
**Theorem 1.2**: *Maximal ideals of $F[x]$ are of the form $\langle f(x) \rangle= \{p(x) \in F[x]: \exists g(x) \in F[x],p(x)=f(x)g(x) \}$ where $f(x)$ is an irreducible polynomial in $F[x]$.*\
 Proof.\
 Proof by contradiction. Assume $f(x)$ is reducible and $\langle f(x) \rangle$ is a maximal ideal of F\[x\].\
 So $\exists g(x),t(x) \in F[x],p(x)=g(x).t(x)$. This means $\langle f(x) \rangle \subset \langle g(x) \rangle \wedge \langle f(x) \rangle \subset \langle t(x) \rangle$.\
 But we assumed that $\langle f(x) \rangle$ is a maximal ideal of F\[x\]. $\bot$

**Theorem 1.3**: *If $I$ is a maximal ideal and $(R,+,.)$ is a commutative ring with unity, then the factor ring $R/I$ is a field.*

**Theorem 1.4**: *Let $I$ be an ideal of $(R,+,.)$, then $(R/I,+^{'},.^{'})$ is a ring.*\
      *$(a+I)+^{'}(b+I)=(a+b)+I$*\
      *$(a+I).^{'}(b+I)=(a.b)+I$*

**Example**: <em>By <em>**Theorem 1.1**</em>, $\mathbb{Z}_{3}[x]$ is an integral domain. Let $f(x)=x^{2}+1$; it is irreducible in $\mathbb{Z}[x]$.</em>\
*$\mathbb{Z}_{3}[x]/ \langle f(x) \rangle= \{\langle f(x) \rangle,(1)+ \langle f(x) \rangle,(2)+ \langle f(x) \rangle,(x)+ \langle f(x) \rangle,(x+1)+ \langle f(x) \rangle,(x+2)+ \langle f(x) \rangle,(2x)+ \langle f(x) \rangle,(2x+1)+ \langle f(x) \rangle,(2x+2)+ \langle f(x) \rangle \}$*\
*$| \mathbb{Z}_{3}[x]/ \langle f(x) \rangle|=9$*

**Definition (Homomorphism)**: *A map $f:(R_{1},+_{1},._{1}) \to(R_{2},+_{2},._{2})$ is a ring homomorphism if $\forall x,y \in R_{1},f(x+_{1}y)=f(x)+_{2}f(y)$ and $f(x._{1}y)=f(x)._{2}f(y)$.*\
**Definition (Isomorphism)**: *A homomorphism $f:(R_{1},+_{1},._{1}) \to(R_{2},+_{2},._{2})$ is a ring isomorphism if $f$ is a one-to-one correspondence.*\
**Theorem 1.5**: *Let $f:(R,+_{1},._{1}) \to(F,+_{2},._{2})$ be an isomorphism. If F is a field, then R is also a field.*\
**Theorem 1.6**: *Let $f:(R_{1},+_{1},._{1}) \to(R_{2},+_{2},._{2})$ be a homomorphism; then $ker(f)= \{x \in R_{1}:f(x)=0_{2} \}$ is an ideal of $R_{1}$.*\
**Theorem 1.7 (First Isomorphism Theorem)**: *Let $f:(R_{1},+_{1},._{1}) \to(R_{2},+_{2},._{2})$ be a homomorphism; then $h:R_{1}/ker(f) \cong Im(f)$ with the rule $h(a+ker(f))=f(a)$.*\
**Definition (Degree)**: *$deg(f(x))$ is the largest $n \in \mathbb{W}$ such that $a_{n} \ne0$.*\
**Theorem 1.8 (Euclid’s Division Lemma)**: *Let $f(x) \ne0,g(x) \in F[x]$, then $\exists q(x),r(x) \in F[x],g(x)=f(x).q(x)+r(x)$ such that $0 \le deg(r(x))<deg(f(x))$*

**Definition (Root)**: *$\omega$ is called a root of the polynomial f(x) iff $f(\omega)=0$ i.e. $\exists g(x) \in F[x],f(x)=(x-\omega)g(x)$*\
**Example**: *Let $f(x)=(-8+11x+-6x^{2}+x^{3}) \in \mathbb{Z}[x]$, then $\omega_{1}=1$, $\omega_{2}=2$ and $\omega_{3}=3$.*

So, let’s play the game.\
**Main Theorem**: *Let $(R,+,.)$ be an integral domain and $p(x)$ be an irreducible polynomial in $(R[x],+,.)$ such that $p(\omega)=0$ then $R[x]/ \langle p(x) \rangle \cong F[\omega]$ and $F[\omega]$ is a field*.\
 Proof.\
      Let $f:(R[x],+,.) \to(R[\omega],+,.)$ be a ring homomorphism.\
      $f(a_{0}+a_{1}x+a_{2}x^{2}+...+a_{n}x^{n})=a_{0}+a_{1} \omega+a_{2} \omega^{2}+...+a_{n} \omega^{n}$\
      By <em>**Theorem 1.6**</em>, $ker(f)= \{q(x) \in R[x]:f(q(x))=0 \}$.\
      By <em>**Theorem 1.8**</em>, $\forall g(x) \in R[x], \exists t(x),r(x) \in R[x],g(x)=p(x).t(x)+r(x)$\
                     $f(g(x))=f(p(x)).f(t(x))+f(r(x))$ by the definition of a homomorphism\
                     $g(\omega)=p(\omega).t(\omega)+r(\omega)$\
                     By assumption, $p(\omega)=0$ then $g(\omega)=0.t(\omega)+r(\omega)$\
                     $g(\omega)=r(\omega)$\
                     This means $f(g(x))=0 \longleftrightarrow r(x)=0$\
      So $ker(f)= \{q(x) \in R[x]: \exists t(x) \in R[x],q(x)=p(x)t(x) \}= \langle p(x) \rangle$\
      By <em>**Theorem 1.7**</em>, $R[x]/ \langle p(x) \rangle \cong Im(f)=f(R[x])=R[\omega]$\
      By <em>**Theorem 1.1**</em>, $R[x]$ is an integral domain and by <em>**Theorem 1.3**</em> $R[x]/ \langle p(x) \rangle$ is a field.\
      By <em>**Theorem 1.5**</em>, $R[\omega]$ is also a field. $\square$

It’s becoming interesting…

**Example**: $(\mathbb{Z}_{5},+,.)$ is a field. Let $p(x)=x^{2}-3$ be an irreducible polynomial in $\mathbb{Z}_{5}[x]$ with $\omega^{2}=3$. By applying the <em>**Main Theorem**</em>, $\mathbb{Z}_{5}[x]/ \langle x^{2}-3 \rangle \cong \mathbb{Z}_{5}[\omega]$.\
$\mathbb{Z}_{5}[x]/ \langle x^{2}-3 \rangle= \{(a+bx)+ \langle x^{2}-3 \rangle:a,b \in \mathbb{Z}_{5} \}$\
$\mathbb{Z}_{5}[\omega]= \{a+b \omega:a,b \in \mathbb{Z}_{5} \}$\
$| \mathbb{Z}_{5}[x]/ \langle x^{2}-3 \rangle|=| \mathbb{Z}_{5}[\omega]|=25$\
 This is because the number of permutations of $a+b \omega$ is 25, since in <em>**Theorem 1.8**</em> we have that $0 \le deg(r(x))<deg(p(x))$.

So let’s generalize for $\mathbb{Z}_{p} \times \mathbb{Z}_{p}$:\
$\mathbb{Z}_{p}$ is a field; let $p(x)$ be an irreducible polynomial in $\mathbb{Z}_{p}[x]$ such that $deg(p(x))=2$ and $p(\omega)=0$.\
 Then, by the <em>**Main Theorem**</em>, $\mathbb{Z}_{p}[x]/ \langle p(x) \rangle \cong \mathbb{Z}_{p}[\omega]$.\
 By **Theorem 1.8**, $| \mathbb{Z}_{p}[\omega]|=p^{2}=| \mathbb{Z}_{p} \times \mathbb{Z}_{p}|$.\
 Let $(a,b)=a+b \omega$; then we can define addition and multiplication on $\mathbb{Z}_{p} \times \mathbb{Z}_{p}$:\
$(a,b)+(c,d)=(a+b \omega)+(c+d \omega)=(a+c)+(b+d) \omega=(a+c,b+d)$\
$(a,b).(c,d)=(a+b \omega).(c+d \omega)=a.c+(a.d) \omega+(b.c) \omega+b.d. \omega^{2}=(a.c+b.d. \omega^{2},a.d+b.c)$

**Question**: How do we know that an irreducible polynomial in $\mathbb{Z}_{p}[x]$ such that $deg(p(x))=2$ exists?\
**Theorem 1.9**: Let (R, +, .) be a ring; then $\forall a,b \in R,(-a).b=a.(-b)=-(a.b)$\
**Theorem 1.10 (Lagrange’s Theorem)**: *Let $G$ be a group and $H \le G$, then $|G|=[G:H].|H|$.*\
**Theorem**: Let $(\mathbb{Z}_{p},+,.)$ be a field. Then there are at least $\lfloor \frac{p}{2} \rfloor$ irreducible polynomials of the form $p(x)$ in $\mathbb{Z}_{p}[x]$ such that $deg(p(x))=2$.\
 Proof.

- $p=2$:\
     There exists only $x^{2}+x+1$.
- $p>2$:\
     Our polynomials must have no root in $\mathbb{Z}_{p}$, because any polynomial with a root is divisible by $(x-\omega)$, which makes it reducible.\
     Let $\mathbb{Z}_{p}^{*}= \{a \in \mathbb{Z}_{p}:gcd(a,p)=1 \}$, so $| \mathbb{Z}_{p}^{*}|= \varphi(p)=p-1$\
     Since the polynomials must have no root in $\mathbb{Z}_{p}$, we remove the integers $a \in \mathbb{Z}_{p}^{*}$ such that $a*a= \omega^{2}$.\
     By <em>**Theorem 1.9**</em>, $a*a=(-a)(-a)= \omega^{2}$. This means there are two <u>different</u> integers in $\mathbb{Z}_{p}^{*}$ that produce $\omega^{2}$ because there is no element of order 2 by <em>**Theorem 1.10**</em>.\
     So there should be $\lfloor \frac{p}{2} \rfloor \omega^{2}$’s to choose from, and the polynomials are of the form $(x^{2}-\omega^{2})$. $\square$
