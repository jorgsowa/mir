===description===
count()/array_key_first() throw TypeError on null, so reaching ANY
comparison result (even one that doesn't determine emptiness) already
excludes null. strlen(null) doesn't throw, so only the proven-non-empty
direction excludes null.
===config===
suppress=UnusedVariable,UnusedParam,PossiblyNullArgument
===file===
<?php
/** @param array|null $arr */
function count_non_empty_strips_null(?array $arr): void {
    if (count($arr) > 0) {
        /** @mir-check $arr is non-empty-array */
        $_ = 1;
    }
}

/** @param array|null $arr */
function count_inconclusive_comparison_still_strips_null(?array $arr): void {
    if (count($arr) < 100) {
        /** @mir-check $arr is array */
        $_ = 1;
    }
}

/** @param array|null $arr */
function array_key_first_strips_null(?array $arr): void {
    if (array_key_first($arr) !== null) {
        /** @mir-check $arr is non-empty-array */
        $_ = 1;
    }
}

/** @param string|null $s */
function strlen_non_empty_strips_null(?string $s): void {
    if (strlen($s) > 0) {
        /** @mir-check $s is non-empty-string */
        $_ = 1;
    }
}

/** @param string|null $s */
function strlen_empty_does_not_strip_null(?string $s): void {
    // strlen(null) returns 0 without throwing, so a null $s also satisfies
    // this branch — unlike count()/array_key_first(), null must survive.
    if (strlen($s) === 0) {
        /** @mir-check $s is string|null */
        $_ = 1;
    }
}
===expect===
