===description===
`TReturn` (the 4th Generator type param) is inferred from the generator's own
`return $expr;` statements, same as a normal function's return type.
===config===
suppress=UnusedVariable,MissingReturnType
===file===
<?php
function gen() {
    yield 1;
    yield 2;
    return "done";
}

$g = gen();
/** @mir-check $g is Generator<int, 1|2, mixed, "done"> */
$_ = 1;
===expect===
