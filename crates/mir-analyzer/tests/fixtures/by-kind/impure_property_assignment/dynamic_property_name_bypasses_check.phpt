===description===
A dynamic property name (`$b->$prop = x`) silently bypassed the purity
check on a parameter write, same gap as the immutable-write check —
falls back to the property expression's own source text as a display
name instead of quietly no-oping.
===config===
suppress=UnusedParam
===file===
<?php
class Bag {
    public int $x = 0;
}

/** @pure */
function mutate(Bag $b, string $prop): void {
    $b->$prop = 5;
}
===expect===
ImpurePropertyAssignment@8:4-8:17: Assigning to property $prop of a parameter in a pure or external-mutation-free context
