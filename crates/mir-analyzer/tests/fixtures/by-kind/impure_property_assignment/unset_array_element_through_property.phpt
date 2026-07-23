===description===
unset($this->arr['key']) / unset($param->arr['key']) -- an array-element
removal reached through a property -- bypassed purity entirely.
analyze_unset_stmt's purity check only matched a DIRECT PropertyAccess
target; an ArrayAccess wrapping one (the array-element form) fell
through to the generic read-oriented existence check with no purity
emission at all.
===file===
<?php
namespace Quux;

class Bag {
    public array $items = ['a' => 1];
}

/** @pure */
function clearInPure(Bag $b): void {
    unset($b->items['a']);
}

class Clearer {
    /** @psalm-external-mutation-free */
    public function clearInMutationFree(Bag $b): void {
        unset($b->items['a']);
    }
}
===expect===
ImpurePropertyAssignment@10:10-10:24: Assigning to property items of a parameter in a pure or external-mutation-free context
ImpurePropertyAssignment@16:14-16:28: Assigning to property items of a parameter in a pure or external-mutation-free context
