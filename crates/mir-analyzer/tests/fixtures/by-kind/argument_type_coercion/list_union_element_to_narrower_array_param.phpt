===description===
A list or shape whose element type is a union of int/string, passed to an array param whose
element type is one of the union's members, may fail at runtime: ArgumentTypeCoercion (Info).
Elements that can never match stay errors.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array<int|string, int> $map */
function takes_map(array $map): void { echo count($map); }
/** @param array<int|string, non-empty-string> $map */
function takes_name_map(array $map): void { echo count($map); }
/** @param list<int> $ids */
function takes_ids(array $ids): void { echo count($ids); }

/**
 * @param list<int|string> $mixed
 * @param list<bool> $flags
 * @param list<string|null> $nullable
 */
function run(array $mixed, int|string $v, array $flags, array $nullable): void {
    /** @mir-check $mixed is list<int|string> */
    takes_map($mixed);
//            ^^^^^^ ArgumentTypeCoercion: Argument $map of takes_map() expects 'array<int|string, int>', got 'list<int|string>' — coercion may fail at runtime
    takes_name_map($mixed);
//                 ^^^^^^ ArgumentTypeCoercion: Argument $map of takes_name_map() expects 'array<int|string, non-empty-string>', got 'list<int|string>' — coercion may fail at runtime
    takes_ids([$v]);
//            ^^^^ ArgumentTypeCoercion: Argument $ids of takes_ids() expects 'list<int>', got 'array{0: int|string}' — coercion may fail at runtime
    takes_map($flags);
//            ^^^^^^ InvalidArgument: Argument $map of takes_map() expects 'array<int|string, int>', got 'list<bool>'
    takes_name_map($nullable);
//                 ^^^^^^^^^ InvalidArgument: Argument $map of takes_name_map() expects 'array<int|string, non-empty-string>', got 'list<string|null>'
}
