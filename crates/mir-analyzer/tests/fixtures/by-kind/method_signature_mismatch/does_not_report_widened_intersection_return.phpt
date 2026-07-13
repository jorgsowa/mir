===description===
FN: atomic_subtype had no (TIntersection, TIntersection) arm, so a valid
covariant-return override narrowing to MORE conjuncts (a subtype) was
falsely flagged as incompatible.
===file===
<?php
interface Boxed {
    public function getBox(): Countable&ArrayAccess;
}
class Impl implements Boxed {
    public function getBox(): Countable&ArrayAccess&Iterator {
        throw new RuntimeException();
    }
}
===expect===
