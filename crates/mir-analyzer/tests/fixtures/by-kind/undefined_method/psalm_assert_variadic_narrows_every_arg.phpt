===description===
A `@psalm-assert-if-true` on a variadic param narrows every trailing
positional arg it swallows, not just the first — arg_for_param_index only
ever resolved a single positional arg for the assertion.
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

function test(mixed $a, mixed $b): void {
    if (all_are_users($a, $b)) {
        $a->missing();
        $b->missing();
    }
}
===expect===
UndefinedMethod@20:8-20:21: Method User::missing() does not exist
UndefinedMethod@21:8-21:21: Method User::missing() does not exist
