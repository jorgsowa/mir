===description===
A `// comment` after an item of a multi-line array shape is not part of the next key.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

/**
 * @psalm-type UserShape = array{
 *   id: int, // user ID
 *   name: string, // user name
 * }
 */
final class User
{
    /** @param UserShape $user */
    public function take(array $user): void {}
}

$user = new User();
$user->take(['id' => 1, 'name' => 'a']);
$user->take(['id' => 1]);
//          ^^^^^^^^^^^ InvalidArgument: Argument $user of take() expects 'array{'id': int, 'name': string}', got 'array{'id': 1}'
$user->take(['id' => 1, 'name' => 2]);
//          ^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $user of take() expects 'array{'id': int, 'name': string}', got 'array{'id': 1, 'name': 2}'
