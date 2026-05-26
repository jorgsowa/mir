===description===
Mismatch docblock native intersection argument
===file===
<?php
interface A {
    function foo(): void;
}
interface B {
}
interface C {
}
/**
 * @param A&C $in
 */
function test(A&B $in): void {
    $in->foo();
}

===expect===
MismatchingDocblockParamType
===ignore===
TODO
