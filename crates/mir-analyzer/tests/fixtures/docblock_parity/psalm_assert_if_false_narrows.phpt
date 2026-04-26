===file===
<?php
class User {
    public function name(): string { return ''; }
}

/**
 * @psalm-assert-if-false User $value
 */
function is_not_user(mixed $value): bool {
    return !($value instanceof User);
}

function test(mixed $value): void {
    if (!is_not_user($value)) {
        $value->name();
        $value->missing();
    }
}
===expect===
UndefinedMethod: Method User::missing() does not exist
