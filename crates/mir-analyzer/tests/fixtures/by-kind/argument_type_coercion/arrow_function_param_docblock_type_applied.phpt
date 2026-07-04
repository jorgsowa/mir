===description===
Arrow-function analogue of closure_param_docblock_type_applied.phpt — a
`@param` docblock preceding `fn(...) => ...` must type its parameter too.
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
class A {}
class B extends A {}

function takesB(B $_b) : void {}

$cb = /** @param A $x */ fn($x) => takesB($x);
===expect===
ArgumentTypeCoercion@7:42-7:44: Argument $_b of takesB() expects 'B', got 'A' — coercion may fail at runtime
