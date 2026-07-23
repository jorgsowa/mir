===description===
`$t->counts['x']++` mutates a parameter's property just as much as
`$t->counts['x'] = ...` would, but unary.rs's `++`/`--` handling only
special-cased a direct `PropertyAccess` operand -- an array-index-into-
property operand had no purity check at all.
===config===
suppress=MissingPropertyType,MixedArrayAccess,MixedAssignment,MixedArgument
===file===
<?php
class Tally {
    public $counts = [];
}

/** @pure */
function bump(Tally $t): void {
    $t->counts['x']++;
}
===expect===
ImpurePropertyAssignment@8:4-8:19: Assigning to property counts of a parameter in a pure or external-mutation-free context
