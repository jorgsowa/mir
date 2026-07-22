===description===
`iterator_count($it)` narrows arrays/lists to non-empty variants the same
way `count()`/`strlen()` already do, for both relational and equality forms.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param list<string> $arr */
function test_iterator_count_greater_than_zero(array $arr): void {
    if (iterator_count($arr) > 0) {
        /** @mir-check $arr is non-empty-list<string> */
        $_ = $arr;
    }
}

/** @param list<string> $arr */
function test_iterator_count_not_identical_zero(array $arr): void {
    if (iterator_count($arr) !== 0) {
        /** @mir-check $arr is non-empty-list<string> */
        $_ = $arr;
    }
}

/** @param list<string> $arr */
function test_zero_not_identical_iterator_count(array $arr): void {
    if (0 !== iterator_count($arr)) {
        /** @mir-check $arr is non-empty-list<string> */
        $_ = $arr;
    }
}

/** @param list<string> $arr */
function test_iterator_count_identical_zero(array $arr): void {
    if (iterator_count($arr) === 0) {
        /** @mir-check $arr is array{} */
        $_ = $arr;
    }
}
===expect===
