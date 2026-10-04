===description===
A bare `array` passed to a param typed as an intersection of arrays (`array<string, mixed>`
plus a shape) may or may not satisfy the shape: ArgumentTypeCoercion (Info). Arrays that
contradict a part (a list, a shape missing a required key) stay errors.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MismatchingDocblockParamType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array<string, mixed>&array{id: int, 'name'?: string} $row */
function takes_row(array $row): void { echo count($row); }

/** @param list<string> $names */
function run(array $any, array $names): void {
    /** @mir-check $any is array<array-key, mixed> */
    takes_row($any);
//            ^^^^ ArgumentTypeCoercion: Argument $row of takes_row() expects 'array<string, mixed>&array{'id': int, 'name'?: string}', got 'array' — coercion may fail at runtime
    takes_row($names);
//            ^^^^^^ InvalidArgument: Argument $row of takes_row() expects 'array<string, mixed>&array{'id': int, 'name'?: string}', got 'list<string>'
    takes_row(['name' => 'a']);
//            ^^^^^^^^^^^^^^^ InvalidArgument: Argument $row of takes_row() expects 'array<string, mixed>&array{'id': int, 'name'?: string}', got 'array{'name': "a"}'
}
===expect===
