===description===
Same array-index-write gap as immutable_property_modification's, for the
@pure/@psalm-external-mutation-free flavors: `$param->items[] = x` mutates
the parameter's property in place, but the array-index write path only
read the property, never running the impure-property-assignment check.
===file===
<?php
namespace Baz;

class Bag {
    /** @var array<int> */
    public array $items = [];
}

/** @pure */
function pushInPure(Bag $b, int $n): void {
    $b->items[] = $n;
}

class Pusher {
    /** @psalm-external-mutation-free */
    public function pushInMutationFree(Bag $b, int $n): void {
        $b->items[] = $n;
    }
}
===expect===
ImpurePropertyAssignment@11:4-11:20: Assigning to property items of a parameter in a pure or external-mutation-free context
ImpurePropertyAssignment@17:8-17:24: Assigning to property items of a parameter in a pure or external-mutation-free context
