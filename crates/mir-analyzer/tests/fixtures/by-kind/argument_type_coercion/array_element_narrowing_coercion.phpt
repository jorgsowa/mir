===description===
An array literal whose elements are broader than the param's element type (int vs positive-int)
may fail at runtime: ArgumentTypeCoercion (Info), not InvalidArgument. Unrelated or nullable
element types stay errors.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param list<positive-int> $ids */
function takes_ids(array $ids): void { echo count($ids); }
/** @param list<non-empty-string> $names */
function takes_names(array $names): void { echo count($names); }
/** @param array{id: positive-int} $row */
function takes_row(array $row): void { echo $row['id']; }

function run(int $i, string $s, ?int $n): void {
    $ids = [$i];
    /** @mir-check $ids is array{0: int} */
    takes_ids($ids);
//            ^^^^ ArgumentTypeCoercion: Argument $ids of takes_ids() expects 'list<positive-int>', got 'array{0: int}' — coercion may fail at runtime
    takes_ids([$i, $i]);
//            ^^^^^^^^ ArgumentTypeCoercion: Argument $ids of takes_ids() expects 'list<positive-int>', got 'array{0: int, 1: int}' — coercion may fail at runtime
    takes_names([$s]);
//              ^^^^ ArgumentTypeCoercion: Argument $names of takes_names() expects 'list<non-empty-string>', got 'array{0: string}' — coercion may fail at runtime
    takes_row(['id' => $i]);
    takes_ids([$s]);
//            ^^^^ InvalidArgument: Argument $ids of takes_ids() expects 'list<positive-int>', got 'array{0: string}'
    takes_ids([$n]);
//            ^^^^ InvalidArgument: Argument $ids of takes_ids() expects 'list<positive-int>', got 'array{0: int|null}'
}
===expect===
