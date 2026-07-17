===description===
count()/strlen() comparisons that prove length === 0 narrow arrays/strings
to their empty variants, symmetric with the non-empty direction.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param list<int>|non-empty-array<string, int> $arr */
function test_count_identical_zero_narrows(array $arr): void {
    if (count($arr) === 0) {
        /** @mir-check $arr is list<int> */
        $_ = $arr;
    }
}

/** @param list<int>|non-empty-array<string, int> $arr */
function test_count_less_than_one_narrows(array $arr): void {
    if (count($arr) < 1) {
        /** @mir-check $arr is list<int> */
        $_ = $arr;
    }
}

/** @param list<int>|non-empty-array<string, int> $arr */
function test_count_less_or_equal_zero_narrows(array $arr): void {
    if (count($arr) <= 0) {
        /** @mir-check $arr is list<int> */
        $_ = $arr;
    }
}

/** @param list<int>|non-empty-array<string, int> $arr */
function test_count_not_greater_than_zero_narrows(array $arr): void {
    if (!(count($arr) > 0)) {
        /** @mir-check $arr is list<int> */
        $_ = $arr;
    }
}

/** @param list<int>|non-empty-array<string, int> $arr */
function test_count_not_greater_or_equal_one_narrows(array $arr): void {
    if (!(count($arr) >= 1)) {
        /** @mir-check $arr is list<int> */
        $_ = $arr;
    }
}

/** @param list<int>|non-empty-array<string, int> $arr */
function test_zero_identical_count_narrows(array $arr): void {
    if (0 === count($arr)) {
        /** @mir-check $arr is list<int> */
        $_ = $arr;
    }
}

/** @param list<int>|non-empty-array<string, int> $arr */
function test_count_not_identical_positive_false_branch_narrows(array $arr): void {
    if (count($arr) !== 3) {
        return;
    }
    /** @mir-check $arr is non-empty-list<int>|non-empty-array<string, int> */
    $_ = $arr;
}

/** @param non-empty-string|numeric-string $s */
function test_strlen_identical_zero_narrows(string $s): void {
    if (strlen($s) === 0) {
        /** @mir-check $s is numeric-string */
        $_ = $s;
    }
}

/** @param non-empty-string|numeric-string $s */
function test_strlen_less_than_one_narrows(string $s): void {
    if (strlen($s) < 1) {
        /** @mir-check $s is numeric-string */
        $_ = $s;
    }
}

// count() already known to be non-empty: the empty branch is dead, but the
// variable's type must not collapse to an empty union.
/** @param non-empty-list<int> $arr */
function test_count_zero_on_already_non_empty_stays_unchanged(array $arr): void {
    if (count($arr) === 0) {
        /** @mir-check $arr is non-empty-list<int> */
        $_ = $arr;
    }
}
===expect===
