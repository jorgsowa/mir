===description===
Passing a parameter's property by reference to a built-in function
(`sort($b->items)`) mutates it exactly as much as an explicit assignment
would, but every by-ref write-back site only ever matched
`ExprKind::Variable` -- a property argument was silently skipped, never
even checked for purity.
===config===
suppress=MissingPropertyType,ImpureFunctionCall,MixedArgument
===file===
<?php
class Box {
    public $items = [];
}

/** @pure */
function normalize(Box $b): void {
    sort($b->items);
}
===expect===
ImpurePropertyAssignment@8:9-8:18: Assigning to property items of a parameter in a pure or external-mutation-free context
