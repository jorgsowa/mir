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
//        ^^^^^^^^^^^^^^ ImpurePropertyAssignment: Assigning to property items of a parameter in a pure or external-mutation-free context
}

class Clearer {
    /** @psalm-external-mutation-free */
    public function clearInMutationFree(Bag $b): void {
        unset($b->items['a']);
//            ^^^^^^^^^^^^^^ ImpurePropertyAssignment: Assigning to property items of a parameter in a pure or external-mutation-free context
    }
}
===expect===
