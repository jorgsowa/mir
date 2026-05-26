===description===
Impure method call
===file===
<?php
namespace Bar;

class A {
    public int $a = 5;

    public function foo() : void {
        $this->a++;
    }
}

/** @pure */
function filterOdd(int $i, A $a) : ?int {
    $a->foo();

    if ($i % 2 === 0 || $a->a === 2) {
        return $i;
    }

    return null;
}
===expect===
ImpureMethodCall
===ignore===
TODO
