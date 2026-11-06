===description===
`covariant`/`contravariant` before a type argument is call-site variance, not part of the type name.
===file===
<?php
class M { public function m(): void {} }
class U extends M { public function u(): void {} }
/** @template TKey of array-key @template TModel of M */
class Coll {
    /** @return TModel|null */
    public function first() { return null; }
}
/** @param Coll<int, covariant M> $r */
function cov($r): void {
    $r->first()?->m();
}
/** @param Coll<int, contravariant U> $r */
function contra($r): void {
    $r->first()?->u();
}
/** @param Coll<int, U> $u */
function pass($u): void {
    cov($u);
}
/** @param Coll<int, string> $s */
function reject($s): void {
    cov($s);
//      ^^ InvalidArgument: Argument $r of cov() expects 'Coll<int, covariant M>', got 'Coll<int, string>'
}
