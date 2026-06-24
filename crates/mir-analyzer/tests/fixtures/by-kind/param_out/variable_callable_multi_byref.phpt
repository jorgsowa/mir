===description===
Variable callable with multiple by-ref out-params: each argument variable
gets the correct type written back after the call.
===config===
suppress=UnusedVariable,MissingClosureReturnType,UnusedParam
===file===
<?php
$fill = function(string &$a, int &$b): void {
    $a = "hello";
    $b = 42;
};

$fill($s, $n);
/** @mir-check $s is string */
$_ = $s;
/** @mir-check $n is int */
$_ = $n;
===expect===
