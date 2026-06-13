===description===
adding a literal to a count() result shifts the range lower bound
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param array<int> $arr */
function test(array $arr): void {
    $n = count($arr) + 5;
    /** @mir-check $n is int<5, max> */
    $_ = $n;
}
===expect===
