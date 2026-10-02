===description===
Relational `count()` comparisons filter closed shapes by their possible length range.
===file===
<?php
/** @param array{0: int}|array{0: int, 1: int}|array{0: int, 1: int, 2: int} $a */
function gt(array $a): int {
    if (count($a) > 2) {
        /** @mir-check $a is array{0: int, 1: int, 2: int} */
        return $a[2];
    }
    return 0;
}

/** @param array{0: int}|array{0: int, 1: int} $a */
function lt(array $a): int {
    if (count($a) < 2) {
        /** @mir-check $a is array{0: int} */
        return $a[0];
    }
    return $a[1];
}

/** @param array{0: int}|array{0: int, 1: int} $a */
function reversed(array $a): int {
    if (2 <= count($a)) {
        return $a[1];
    }
    return 0;
}
===expect===
