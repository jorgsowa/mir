===description===
$x =& $this->prop; then a later plain $x = value; mutates the property
through the reference, but purity checks never tracked reference
aliasing at all -- only a direct $this->prop = value write was caught.
Narrow: only this one AST-visible pattern (a bare local var ref-aliased
directly to a var-receiver property) is tracked.
===config===
suppress=UnusedVariable
===file===
<?php
class Bag {
    public int $x = 1;
}

/** @pure */
function mutate(Bag $b): void {
    $ref = &$b->x;
    $ref = 5;
}
===expect===
ImpurePropertyAssignment@9:4-9:12: Assigning to property x of a parameter in a pure or external-mutation-free context
