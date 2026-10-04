===description===
Arguments of `$var::method(...)` on a receiver typed as a union of classes
are still read, so their variables are not reported unused.
===file===
<?php
class A {
    public static function make(int $n): int { return $n; }
}
class B {
    public static function make(int $n): int { return $n * 2; }
}

function unionReceiver(A|B $c): void {
    $arg = 5;
    $c::make($arg);
}

function unionReceiverNested(A|B $c): void {
    $a = 1;
    $b = 2;
    $c::make($a + $b);
}

function classStringUnion(string $s): void {
    $arg = 5;
    $s::make($arg);
}
===expect===
