===file===
<?php
/**
 * @mixin C
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

/**
 * @mixin B
 */
class C {
    public function fromC(): string { return ''; }
}

function test(A $a): void {
    $a->fromA();
}
===expect===
