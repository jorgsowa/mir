===file===
<?php
class User {
    public function name(): string { return ''; }
}

/**
 * @phpstan-assert-if-true User $value
 */
function is_user(mixed $value): bool {
    return $value instanceof User;
}

function test(mixed $value): void {
    if (is_user($value)) {
        $value->name();
        $value->missing();
    }
}
===expect===
UndefinedMethod: Method User::missing() does not exist
