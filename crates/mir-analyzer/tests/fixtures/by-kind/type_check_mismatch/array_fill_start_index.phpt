===description===
`array_fill_return_type` never read `$start_index` (1st arg) — a nonzero
start (or a negative one, where only the first key keeps it and the rest
restart from 0) was still typed as `non-empty-list`, even though PHP only
produces a list when `$start_index` is exactly 0.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function fillFromZeroIsList(): void {
    $r = array_fill(0, 3, 'x');
    /** @mir-check $r is non-empty-list<"x"> */
    $_ = $r;
}

function fillFromPositiveStartIsNotList(): void {
    $r = array_fill(5, 3, 'x');
    /** @mir-check $r is non-empty-array<int, "x"> */
    $_ = $r;
}

function fillFromNegativeStartIsNotList(): void {
    $r = array_fill(-2, 3, 'x');
    /** @mir-check $r is non-empty-array<int, "x"> */
    $_ = $r;
}
===expect===
