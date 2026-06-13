===description===
count() over a possibly-empty array is int<0, max>
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param array<string, int> $arr */
function test(array $arr): void {
    $n = count($arr);
    /** @mir-check $n is int<0, max> */
    $_ = $n;
}
===expect===
