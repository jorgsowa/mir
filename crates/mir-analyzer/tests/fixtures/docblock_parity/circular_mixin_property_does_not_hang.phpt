===file===
<?php
/**
 * @mixin B
 */
class A {
    public string $fromA = '';
}

/**
 * @mixin A
 */
class B {
    public string $fromB = '';
}

function test(A $a): void {
    strlen($a->fromA);
}
===expect===
