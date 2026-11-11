===description===
`literal-string` and `literal-int` still reject values of another type, and `non-empty-literal-string` rejects the empty literal.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param literal-string $name */
function column(string $name): void {}

/** @param non-empty-literal-string $name */
function table(string $name): void {}

/** @param literal-int $count */
function repeat(int $count): void {}

column('id');
column(['id']);
//     ^^^^^^ InvalidArgument: Argument $name of column() expects 'literal-string', got 'array{0: "id"}'
table('users');
table('');
//    ^^ InvalidArgument: Argument $name of table() expects 'non-empty-literal-string', got '""'
repeat(3);
repeat('three');
//     ^^^^^^^ InvalidArgument: Argument $count of repeat() expects 'literal-int', got '"three"'
repeat(null);
//     ^^^^ NullArgument: Argument $count of repeat() cannot be null
