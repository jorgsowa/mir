===description===
A function with no return type declaration at all is inferred as
Generator<TKey, TValue, mixed, void> from its `$k => $v` yields.
===config===
suppress=UnusedVariable,MissingReturnType
===file===
<?php
function gen() {
    yield 'a' => 1;
    yield 'b' => 2;
}

$g = gen();
/** @mir-check $g is Generator<"a"|"b", 1|2, mixed, void> */
$_ = 1;
===expect===
