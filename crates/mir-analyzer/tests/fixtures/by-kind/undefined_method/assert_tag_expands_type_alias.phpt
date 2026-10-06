===description===
An assertion tag's type never got local type-alias expansion, unlike
@param/@return right next to it -- an alias-named assertion type stayed
an unresolved, unexpandable bare atom instead of narrowing to the real
class it stands for.
===config===
<mir>
  <issueHandlers>
    <MissingParamType errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//  ^^^^^^^^^^^^^^^^^ UndefinedMethod: Method User::missing() does not exist
}
