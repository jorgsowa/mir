===description===
A shape with a nullable value passed where the shape's value must not be null is `PossiblyInvalidArgument`, as it is for `list<string>`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

/** @param list{string} $list */
function takesStringList(array $list): void {}

/** @param array{name: string} $row */
function takesRow(array $row): void {}

/** @param list{string|null} $list */
function takesNullableList(array $list): void {}

function unguarded(?string $value): void {
    $list = [$value];
    takesStringList($list);
    //              ^^^^^ PossiblyInvalidArgument: Argument $list of takesStringList() expects 'array{0: string}', possibly different type 'array{0: string|null}' provided
    takesRow(['name' => $value]);
    //       ^^^^^^^^^^^^^^^^^^ PossiblyInvalidArgument: Argument $row of takesRow() expects 'array{'name': string}', possibly different type 'array{'name': string|null}' provided
    takesNullableList($list);
}

function guarded(?string $value): void {
    if ($value !== null) {
        takesStringList([$value]);
    }
}
