===description===
count() - 1 can be -1 (empty array) — the range lower bound drops to -1
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param array<int> $arr */
function test(array $arr): void {
    $last = count($arr) - 1;
    /** @mir-check $last is int<-1, max> */
    $_ = $last;
}
===expect===
