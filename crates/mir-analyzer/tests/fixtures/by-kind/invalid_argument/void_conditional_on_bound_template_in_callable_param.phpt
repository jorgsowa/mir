===description===
`(T is void ? null : T)` resolves to the bound type when `T` is bound to a non-void type.
===file===
<?php
/** @template T */
interface Pr {
    /** @template TF @param ?(callable((T is void ? null : T)): TF) $onFulfilled @return Pr<TF> */
    public function then(?callable $onFulfilled = null): Pr;
}
final class Resp {}
/** @param Pr<Resp> $p */
function f(Pr $p): void { $p->then(static function (Resp $r): void {}); }
