===description===
`yield $v;` without an explicit key defaults to an int key, same as PHP's
own auto-incrementing generator keys.
===config===
suppress=UnusedVariable,MissingReturnType
===file===
<?php
function gen() {
    yield "x";
    yield "y";
}

$g = gen();
/** @mir-check $g is Generator<int, string, mixed, void> */

===expect===
