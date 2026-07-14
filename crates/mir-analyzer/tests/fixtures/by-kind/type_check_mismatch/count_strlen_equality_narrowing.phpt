===description===
count()/strlen() equality comparisons (===0, !==0, ==0, !=0, exact positive
counts) narrow arrays/strings to non-empty variants, like the </>/<=/>= forms.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param list<string> $arr */
function test_count_not_identical_zero(array $arr): void {
    if (count($arr) !== 0) {
        /** @mir-check $arr is non-empty-list<string> */
        $_ = $arr;
    }
}

/** @param list<string> $arr */
function test_count_not_equal_zero(array $arr): void {
    if (count($arr) != 0) {
        /** @mir-check $arr is non-empty-list<string> */
        $_ = $arr;
    }
}

/** @param list<string> $arr */
function test_count_identical_zero_false_branch(array $arr): void {
    if (count($arr) === 0) {
        return;
    }
    /** @mir-check $arr is non-empty-list<string> */
    $_ = $arr;
}

/** @param list<string> $arr */
function test_zero_not_identical_count(array $arr): void {
    if (0 !== count($arr)) {
        /** @mir-check $arr is non-empty-list<string> */
        $_ = $arr;
    }
}

/** @param list<string> $arr */
function test_count_identical_exact_positive(array $arr): void {
    if (count($arr) === 3) {
        /** @mir-check $arr is non-empty-list<string> */
        $_ = $arr;
    }
}

function test_strlen_not_identical_zero(string $s): void {
    if (strlen($s) !== 0) {
        /** @mir-check $s is non-empty-string */
        $_ = $s;
    }
}

function test_strlen_identical_zero(string $s): void {
    if (strlen($s) === 0) {
        /** @mir-check $s is string */
        $_ = $s;
    }
}
===expect===
