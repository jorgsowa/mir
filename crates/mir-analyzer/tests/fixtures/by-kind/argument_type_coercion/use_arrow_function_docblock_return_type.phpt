===description===
Use arrow function docblock return type
Arrow-function analogue of use_closure_docblock_type.phpt — a `@return` docblock
immediately preceding `fn(...) => ...` must override the inferred return type,
just like it does for `function(...) {...}`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {}
class B extends A {}

function takesA(A $_a) : void {}
function takesB(B $_b) : void {}

$getAButReallyB = /** @return A */ fn() => new B;

takesA($getAButReallyB());
takesB($getAButReallyB());
//     ^^^^^^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $_b of takesB() expects 'B', got 'A' — coercion may fail at runtime
