===description===
`array_is_list()` narrowing now recognizes literal/shape `TKeyedArray`s (via
their own `is_list` flag) instead of always treating them as non-list.
`array_key_exists()` proving a new key present only keeps a shape's
`is_list` flag when the new key continues the sequence — a string key (or
any non-contiguous int) correctly clears it, so a later `array_is_list()`
check is proven false rather than wrongly still-possibly-true.
===config===
suppress=UnusedVariable,MissingConstructor
===file===
<?php
function plainListLiteralStaysList(): void {
    $arr = [1, 2, 3];
    if (array_is_list($arr)) {
//      ^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition of type 'bool' always evaluates the same way, so one branch is unreachable
        /** @mir-check $_ is never */
        $_ = 1;
//      ^^^^^^^ TypeCheckMismatch: Type of $_ is expected to be never, got mixed
    }
}

function keyExistsStringKeyBreaksListNarrowing(): void {
    $arr = [1, 2, 3];
    if (array_key_exists('foo', $arr)) {
        if (array_is_list($arr)) {
//          ^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition of type 'bool' always evaluates the same way, so one branch is unreachable
            /** @mir-check $_ is never */
            $_ = 1;
        }
    }
}

function keyExistsContiguousIntKeyPreservesList(): void {
    $arr = [1, 2, 3];
    if (array_key_exists(3, $arr)) {
        if (array_is_list($arr)) {
//          ^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition of type 'bool' always evaluates the same way, so one branch is unreachable
            /** @mir-check $_ is never */
            $_ = 1;
//          ^^^^^^^ TypeCheckMismatch: Type of $_ is expected to be never, got mixed
        }
    }
}
===expect===
