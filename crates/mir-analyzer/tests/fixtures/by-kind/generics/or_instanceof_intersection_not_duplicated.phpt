===description===
`$x instanceof C || $x instanceof D` narrowing a union that includes an
unrelated intersection member (`(A&B)|C|D`) must not duplicate that
intersection once per disjunct. Before this fix the narrowed type was
`A&B&C|C|A&B&D|D` (the `A&B` leg bolted onto both C and D separately,
double-counting the same ambiguity); it is now `A&B&(C|D)|C|D` — one
intersection branch capturing both remaining possibilities, not two. This
is still not folded down to the fully-simplified `C|D` (a value already
known to satisfy the intersection branch is provably also `C|D`), which
would need a general union-simplification pass — pinning the improved,
not-yet-fully-simplified type so a future simplification pass updates this
fixture deliberately.
===config===
suppress=UnusedVariable
===file===
<?php
interface A {}
interface B {}
class C {}
class D {}

/** @param (A&B)|C|D $x */
function f($x): void {
    if ($x instanceof C || $x instanceof D) {
        /** @mir-check $x is C|D */
        $_ = 1;
    }
}
===expect===
TypeCheckMismatch@11:8-11:15: Type of $x is expected to be C|D, got A&B&C|D|C|D
