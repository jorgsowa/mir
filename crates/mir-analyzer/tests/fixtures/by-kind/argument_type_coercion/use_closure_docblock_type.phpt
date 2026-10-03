===description===
Use closure docblock type
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

$getAButReallyB = /** @return A */ function() {
    return new B;
};

takesA($getAButReallyB());
takesB($getAButReallyB());
//     ^^^^^^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $_b of takesB() expects 'B', got 'A' — coercion may fail at runtime
===expect===
