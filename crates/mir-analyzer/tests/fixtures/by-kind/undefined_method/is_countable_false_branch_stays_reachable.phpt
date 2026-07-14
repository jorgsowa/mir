===description===
Same as `is_iterable_false_branch_stays_reachable.phpt` but for
`is_countable()`.
===file===
<?php
class Box {}

/** @param Box|array<int,int> $x */
function f($x): void {
    if (is_countable($x)) {
        count($x);
    } else {
        $x->method();
    }
}
===expect===
PossiblyInvalidArgument@7:14-7:16: Argument $value of count() expects 'array|Countable', possibly different type 'Box|array<int, int>' provided
UndefinedMethod@9:8-9:20: Method Box::method() does not exist
