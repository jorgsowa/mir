===file===
<?php
/**
 * @mixin A
 */
class A {
    public function foo(): string { return ''; }
}

function test(A $a): void {
    $a->foo();
}
===expect===
