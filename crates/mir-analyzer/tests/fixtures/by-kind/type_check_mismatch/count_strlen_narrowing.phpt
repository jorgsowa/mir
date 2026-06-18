===description===
count() > 0 and strlen() > 0 narrow arrays/strings to non-empty variants.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param list<string> $arr */
function test_count_gt_zero(array $arr): void {
    if (count($arr) > 0) {
        /** @mir-check $arr is non-empty-list<string> */
        $_ = $arr;
    }
}

/** @param list<string> $arr */
function test_count_gte_one(array $arr): void {
    if (count($arr) >= 1) {
        /** @mir-check $arr is non-empty-list<string> */
        $_ = $arr;
    }
}

/** @param array<string, int> $arr */
function test_count_array_gt_zero(array $arr): void {
    if (count($arr) > 0) {
        /** @mir-check $arr is non-empty-array<string, int> */
        $_ = $arr;
    }
}

function test_strlen_gt_zero(string $s): void {
    if (strlen($s) > 0) {
        /** @mir-check $s is non-empty-string */
        $_ = $s;
    }
}

function test_strlen_gte_one(string $s): void {
    if (strlen($s) >= 1) {
        /** @mir-check $s is non-empty-string */
        $_ = $s;
    }
}
===expect===
