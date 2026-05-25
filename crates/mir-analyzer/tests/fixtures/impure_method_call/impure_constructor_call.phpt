===description===
impureConstructorCall
===file===
<?php
namespace Bar;

class A {
    public int $a = 5;
}

class B {
    public function __construct(A $a) {
        $a->a++;
    }
}

/** @pure */
function filterOdd(int $i, A $a) : ?int {
    $b = new B($a);

    if ($i % 2 === 0 || $a->a === 2) {
        return $i;
    }

    return null;
}
===expect===
ImpureMethodCall
===ignore===
TODO
