===description===
`@phan-type` declares a local type alias like `@psalm-type` and `@phpstan-type`.
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
 * @phan-type UserRow = array{id: int, name: string}
 */
final class Repository
{
    /** @param UserRow $row */
    public function save($row): void {}
}

$repository = new Repository();
$repository->save(['id' => 1, 'name' => 'Ada']);
$repository->save(['id' => 1]);
//                ^^^^^^^^^^^ InvalidArgument: Argument $row of save() expects 'array{'id': int, 'name': string}', got 'array{'id': 1}'
