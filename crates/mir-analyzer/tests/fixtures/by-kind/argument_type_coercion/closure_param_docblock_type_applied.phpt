===description===
A `@param` docblock immediately preceding a closure literal must type its
parameter for checks inside the closure body — previously only native type
hints were used, so a docblock-only param type silently stayed `mixed`.
===config===
suppress=UnusedParam,UnusedVariable,MissingClosureReturnType
===file===
<?php
class A {}
class B extends A {}

function takesB(B $_b) : void {}

$cb = /** @param A $x */ function($x) {
    takesB($x);
};
===expect===
ArgumentTypeCoercion@8:11-8:13: Argument $_b of takesB() expects 'B', got 'A' — coercion may fail at runtime
