===description===
`array_reverse_return_type` unconditionally returned a list type, ignoring
the source's actual key type and the `$preserve_keys` argument. PHP only
renumbers INT keys when `$preserve_keys` is false (the default); string
keys are always kept, and `$preserve_keys = true` keeps int keys too.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param array<string, int> $arr */
function reverseStringKeyedStaysArray(array $arr): void {
    $r = array_reverse($arr);
    /** @mir-check $r is array<string, int> */
    $_ = $r;
}

/** @param non-empty-list<int> $arr */
function reverseListDefaultStaysList(array $arr): void {
    $r = array_reverse($arr);
    /** @mir-check $r is non-empty-list<int> */
    $_ = $r;
}

/** @param non-empty-list<int> $arr */
function reversePreserveKeysTrueKeepsIntKeys(array $arr): void {
    $r = array_reverse($arr, true);
    /** @mir-check $r is non-empty-array<int, int> */
    $_ = $r;
}
===expect===
