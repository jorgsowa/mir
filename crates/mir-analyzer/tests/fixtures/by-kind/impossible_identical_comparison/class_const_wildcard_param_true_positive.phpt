===description===
A comparison against a constant outside the wildcard set is still reported.
===file===
<?php
class D {
    const MODE_KEEP = 1;
    const MODE_DROP = 2;
    const OTHER = 9;

    /** @param self::MODE_* $mode */
    public function f(int $mode): bool {
        return $mode === self::OTHER;
//             ^^^^^^^^^^^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between '1|2' and '9' is always false — these types can never be identical
    }
}
