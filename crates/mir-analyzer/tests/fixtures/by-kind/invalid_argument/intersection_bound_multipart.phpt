===description===
class implementing three parts of an intersection bound should pass
===file===
<?php
interface A {}
interface B {}
interface C {}

class Impl implements A, B, C {}

/**
 * @template T of A&B&C
 */
function multi(A&B&C $t): void {
    echo (string) $t;
}

multi(new Impl());
===expect===
