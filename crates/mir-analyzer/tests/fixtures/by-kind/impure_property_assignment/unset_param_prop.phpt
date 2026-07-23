===description===
Same unset() gap as immutable_property_modification's, for the
@pure/@psalm-external-mutation-free flavors: `unset($param->prop)` mutates
the parameter's property, but analyze_unset_stmt only did a read-oriented
existence check, never running the impure-property-assignment check.
===file===
<?php
namespace Qux;

class Bag {
    public ?int $x = 1;
}

/** @pure */
function clearInPure(Bag $b): void {
    unset($b->x);
}

class Clearer {
    /** @psalm-external-mutation-free */
    public function clearInMutationFree(Bag $b): void {
        unset($b->x);
    }
}
===expect===
ImpurePropertyAssignment@10:10-10:15: Assigning to property x of a parameter in a pure or external-mutation-free context
ImpurePropertyAssignment@16:14-16:19: Assigning to property x of a parameter in a pure or external-mutation-free context
