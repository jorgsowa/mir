===file===
<?php
class User {
    public function name(): string { return ''; }
}

/**
 * @psalm-assert User $value
 */
function assert_user(mixed $value): void {
    if (!$value instanceof User) {}
}

function test(mixed $value): void {
    assert_user($value);
    $value->name();
    $value->missing();
}
===expect===
UndefinedMethod: Method User::missing() does not exist
