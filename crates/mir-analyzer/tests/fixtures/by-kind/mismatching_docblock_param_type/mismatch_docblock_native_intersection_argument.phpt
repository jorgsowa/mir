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
MismatchingDocblockParamType@12:19-12:22: Docblock type 'A&C' for $in does not match inferred 'A&B'
