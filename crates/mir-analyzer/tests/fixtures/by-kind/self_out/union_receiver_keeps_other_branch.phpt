===description===
When only one branch of a union receiver declares @psalm-self-out, the
other branch's type must survive in the retyped union — not get silently
overwritten by whichever atomic happened to be resolved last.
===config===
suppress=UnusedParam
===file===
<?php
class A {
    /** @psalm-self-out AReady */
    public function touch(): void {}
}
class AReady extends A {}

class B {
    public function touch(): void {}
}

/** @param A|B $x */
function test($x): void {
    $x->touch();
    /** @mir-check $x is AReady|B */
    $_ = 1;
}
===expect===
