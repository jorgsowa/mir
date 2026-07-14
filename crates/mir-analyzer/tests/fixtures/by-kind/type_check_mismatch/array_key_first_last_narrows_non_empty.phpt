===description===
array_key_first($arr)/array_key_last($arr) !== null narrows $arr to
non-empty-array, same non-empty idiom as count($arr) > 0.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param array<string, int> $arr */
function test_key_first_not_null(array $arr): void {
    if (array_key_first($arr) !== null) {
        /** @mir-check $arr is non-empty-array<string, int> */
        $_ = $arr;
    }
}

/** @param array<string, int> $arr */
function test_key_last_not_null_reversed(array $arr): void {
    if (null !== array_key_last($arr)) {
        /** @mir-check $arr is non-empty-array<string, int> */
        $_ = $arr;
    }
}

/** @param array<string, int> $arr */
function test_key_first_null_not_narrowed(array $arr): void {
    if (array_key_first($arr) === null) {
        /** @mir-check $arr is array<string, int> */
        $_ = $arr;
    }
}
===expect===
