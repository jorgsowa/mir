===description===
`contravariant X` accepts `X` or a supertype and `*` any argument; reading through either gives the template's bound.
===file===
<?php
class M { public function m(): void {} }
class U extends M {}
class Other {}
/** @template TModel of M */
class Coll {
    /** @return TModel|null */
    public function first() { return null; }
}
/** @param Coll<contravariant U> $r */
function contra($r): void {
    $r->first()?->m();
}
/** @param Coll<*> $r */
function star($r): void {
    $r->first()?->m();
}
/**
 * @param Coll<M> $m
 * @param Coll<U> $u
 * @param Coll<Other> $o
 */
function pass($m, $u, $o): void {
    contra($m);
    contra($u);
    star($o);
    contra($o);
//         ^^ InvalidArgument: Argument $r of contra() expects 'Coll<contravariant U>', got 'Coll<Other>'
}
