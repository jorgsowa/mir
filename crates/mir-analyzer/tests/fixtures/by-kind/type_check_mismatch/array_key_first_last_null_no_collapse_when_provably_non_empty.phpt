===description===
array_key_first($arr)/array_key_last($arr) === null on an array already
known to be exclusively non-empty must not collapse $arr to an empty
union — same no-collapse guard count()/strlen() comparisons already have.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param non-empty-array<string, int> $arr */
function test_key_first_null_on_already_non_empty(array $arr): void {
    if (array_key_first($arr) === null) {
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'int|string' and 'null' is always false — these types can never be identical
        /** @mir-check $arr is non-empty-array<string, int> */
        $_ = $arr;
    }
}

/** @param non-empty-array<string, int> $arr */
function test_key_last_null_on_already_non_empty(array $arr): void {
    if (null === array_key_last($arr)) {
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'null' and 'int|string' is always false — these types can never be identical
        /** @mir-check $arr is non-empty-array<string, int> */
        $_ = $arr;
    }
}
===expect===
