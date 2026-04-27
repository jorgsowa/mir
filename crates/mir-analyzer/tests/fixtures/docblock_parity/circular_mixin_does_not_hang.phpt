===file===
<?php
/**
 * @mixin B
 */
class A {
    public function fromA(): string { return ''; }
}

/**
 * @mixin A
 */
class B {
    public function fromB(): string { return ''; }
}

function test(A $a, B $b): void {
    $a->fromA();
    $b->fromB();
}
===expect===
