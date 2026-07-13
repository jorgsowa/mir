===description===
FN: atomic_subtype had no (TIntersection, TIntersection) arm, so a valid
contravariant-param override widening to FEWER conjuncts (a supertype)
was falsely flagged as narrowing.
===config===
suppress=UnusedParam
===file===
<?php
interface Boxed {
    public function setBox(Countable&ArrayAccess&Iterator $box): void;
}
class Impl implements Boxed {
    public function setBox(Countable&ArrayAccess $box): void {}
}
===expect===
