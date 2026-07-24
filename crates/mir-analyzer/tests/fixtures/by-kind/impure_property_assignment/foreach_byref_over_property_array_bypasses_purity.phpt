===description===
`foreach ($t->items as &$v)` mutates that property's array contents in
place, exactly as much as a by-ref call argument (`sort($t->items)`,
already checked) does — but the foreach statement never routed the
iterable expression through the same purity check at all.
===config===
suppress=MissingPropertyType,UnusedForeachValue,MixedAssignment
===file===
<?php
class Tally {
    public $items = [1, 2, 3];
}

/** @pure */
function bumpAll(Tally $t): void {
    foreach ($t->items as &$v) {
        $v++;
    }
}
===expect===
ImpurePropertyAssignment@8:13-8:22: Assigning to property items of a parameter in a pure or external-mutation-free context
