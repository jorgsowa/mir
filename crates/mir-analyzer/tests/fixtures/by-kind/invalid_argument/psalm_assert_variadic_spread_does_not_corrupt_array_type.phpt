===description===
A `@psalm-assert-if-true` on a variadic param, called via a spread
argument (`f(...$list)`), resolved the single spread `Arg`'s value (the
WHOLE array) as if it were one of the variadic's own scalar elements —
overwriting the array variable's own tracked type with the assertion's
per-element type. `count($list)` on the (correctly still-array) `$list`
must not be flagged.
===config===
suppress=MixedAssignment
===file===
<?php
class User {
    public function name(): string { return ''; }
}

/**
 * @psalm-assert-if-true User $values
 */
function all_are_users(mixed ...$values): bool {
    foreach ($values as $v) {
        if (!($v instanceof User)) {
            return false;
        }
    }
    return true;
}

/**
 * @param list<mixed> $list
 */
function test(array $list): void {
    if (all_are_users(...$list)) {
        echo count($list);
    }
}
===expect===
