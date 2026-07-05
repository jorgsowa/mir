===description===
`yield from $anotherGenerator` contributes the delegated generator's own
key/value type params, propagating through generator delegation chains.
===config===
suppress=UnusedVariable,MissingReturnType
===file===
<?php
function inner() {
    yield 'k' => 1;
}

function outer() {
    yield from inner();
}

$g = outer();
/** @mir-check $g is Generator<string, int, mixed, void> */

===expect===
