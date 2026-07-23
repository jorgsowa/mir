===description===
Passing an array-index-into-property expression by reference to a
built-in function (`sort($b->buckets['x'])`) mutates that property's
contents just as much as a direct property argument (`sort($b->items)`,
already fixed) does, but `check_byref_arg_purity` only special-cased a
direct `PropertyAccess` argument, not an `ArrayAccess` whose base is one.
===config===
suppress=MissingPropertyType,ImpureFunctionCall,MixedArgument,MixedArrayAccess,MixedAssignment
===file===
<?php
class Bag {
    public $buckets = [];
}

/** @pure */
function normalize(Bag $b): void {
    sort($b->buckets['x']);
}
===expect===
ImpurePropertyAssignment@8:9-8:25: Assigning to property buckets of a parameter in a pure or external-mutation-free context
