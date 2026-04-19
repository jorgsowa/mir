===source===
<?php
// Bug: elseif branches were not narrowed on the if condition being false — the
// if condition's type exclusion was not applied before entering the elseif body.
// Here $x === null is handled by the if, so in the elseif $x can only be string;
// the is_string() check is therefore redundant.
function foo(string|null $x): void {
    if ($x === null) {
        // $x is null
    } elseif (is_string($x)) {
        // $x is already string (null excluded by the if above)
    }
}
===expect===
RedundantCondition: Condition is always true/false for type 'bool'
