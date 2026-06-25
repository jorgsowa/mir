===description===
Impure property assignment
===file===
<?php
namespace Bar;

class A {
    public int $a = 5;
}

/** @pure */
function filterOdd(int $i, A $a) : ?int {
    $a->a = $i;

    if ($i % 2 === 0 || $a->a === 2) {
        return $i;
    }

    return null;
}
===expect===
ImpurePropertyAssignment@10:4-10:14: Assigning to property a of a parameter in a pure or external-mutation-free context
