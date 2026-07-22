===description===
array_key_first($arr)/array_key_last($arr) === null narrows $arr to the
closed empty shape `array{}` — it drops the non-empty-array/non-empty-list
variants (since those can never be empty) and narrows the remaining plain
array/list down to the same type an empty `[]` literal itself has.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param non-empty-array<string, int>|list<int> $arr */
function test_key_first_null_drops_non_empty(array $arr): void {
    if (array_key_first($arr) === null) {
        /** @mir-check $arr is array{} */
        $_ = $arr;
    }
}

/** @param array<string, int> $arr */
function test_key_first_null_keeps_plain_array(array $arr): void {
    if (array_key_first($arr) === null) {
        /** @mir-check $arr is array{} */
        $_ = $arr;
    }
}

/** @param non-empty-array<string, int>|list<int> $arr */
function test_key_last_null_reversed(array $arr): void {
    if (null === array_key_last($arr)) {
        /** @mir-check $arr is array{} */
        $_ = $arr;
    }
}
===expect===
