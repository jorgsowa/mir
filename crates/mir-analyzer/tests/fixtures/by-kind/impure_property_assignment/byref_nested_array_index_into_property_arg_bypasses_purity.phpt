===description===
A TWO-level array-index-into-property expression by reference
(`sort($b->buckets['x']['y'])`) mutates that property's contents just as
much as a single-level index (`sort($b->buckets['x'])`, already fixed)
does, but `check_byref_arg_purity`'s `ArrayAccess` arm only ever unwrapped
ONE level before checking the base, silently skipping deeper nesting.
===config===
suppress=MissingPropertyType,ImpureFunctionCall,MixedArgument,MixedArrayAccess,MixedAssignment
===file===
<?php
class Bag {
    public $buckets = [];
}

/** @pure */
function normalize(Bag $b): void {
    sort($b->buckets['x']['y']);
}
===expect===
ImpurePropertyAssignment@8:9-8:30: Assigning to property buckets of a parameter in a pure or external-mutation-free context
