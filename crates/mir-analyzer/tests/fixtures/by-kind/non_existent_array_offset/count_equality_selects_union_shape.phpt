===description===
`count($a) === N` drops closed shapes that can't have N entries, in both branches.
===file===
<?php
/** @param array{0: int}|array{0: int, 1: int} $a */
function eq(array $a): int {
    if (count($a) === 2) {
        /** @mir-check $a is array{0: int, 1: int} */
        return $a[1];
    }
    /** @mir-check $a is array{0: int} */
    return $a[0];
}

/** @param array{0: int}|array{0: int, 1: int} $a */
function ne(array $a): int {
    if (count($a) !== 1) {
        return $a[1];
    }
    return $a[0];
}

/** @param array{0: int}|array{0: int, 1: int} $a */
function noGuard(array $a): int {
    return $a[1];
//            ^ NonExistentArrayOffset: Array offset '1' does not exist
//  ^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'int'
}
===expect===
