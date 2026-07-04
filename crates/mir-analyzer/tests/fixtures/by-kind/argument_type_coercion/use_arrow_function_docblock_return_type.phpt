===description===
Use arrow function docblock return type
Arrow-function analogue of use_closure_docblock_type.phpt — a `@return` docblock
immediately preceding `fn(...) => ...` must override the inferred return type,
just like it does for `function(...) {...}`.
===config===
suppress=UnusedParam
===file===
<?php
class A {}
class B extends A {}

function takesA(A $_a) : void {}
function takesB(B $_b) : void {}

$getAButReallyB = /** @return A */ fn() => new B;

takesA($getAButReallyB());
takesB($getAButReallyB());
===expect===
ArgumentTypeCoercion@11:7-11:24: Argument $_b of takesB() expects 'B', got 'A' — coercion may fail at runtime
