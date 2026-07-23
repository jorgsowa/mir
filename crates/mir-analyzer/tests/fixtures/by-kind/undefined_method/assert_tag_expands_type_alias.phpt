===description===
An assertion tag's type never got local type-alias expansion, unlike
@param/@return right next to it -- an alias-named assertion type stayed
an unresolved, unexpandable bare atom instead of narrowing to the real
class it stands for.
===config===
suppress=MissingParamType,MissingReturnType,UnusedParam
===file===
<?php
class User {
    public function name(): string { return ''; }
}

/**
 * @psalm-type UserAlias = User
 * @psalm-assert UserAlias $value
 */
function assertIsUser($value): void {}

function process($value): void {
    assertIsUser($value);
    $value->name();
    $value->missing();
}
===expect===
UndefinedMethod@15:4-15:21: Method User::missing() does not exist
