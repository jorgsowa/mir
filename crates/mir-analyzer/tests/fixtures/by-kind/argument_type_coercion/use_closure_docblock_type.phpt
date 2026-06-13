===description===
Use closure docblock type
===config===
suppress=UnusedParam
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
===expect===
ArgumentTypeCoercion@13:8-13:25: Argument $_b of takesB() expects 'B', got 'A' — coercion may fail at runtime
