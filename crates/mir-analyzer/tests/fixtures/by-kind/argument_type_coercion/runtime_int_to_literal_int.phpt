===description===
An int computed at runtime is a coercion to `literal-int`; int literals and constants are accepted.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Paging {
    public const SIZE = 25;
}

/** @param literal-int $size */
function page(int $size): void {}

/** @param list<string> $rows */
function run(int $requested, array $rows, bool $wide): void {
    page(10);
    page(-1);
    page(Paging::SIZE);
    page($wide ? 50 : 25);
    page($requested);
//       ^^^^^^^^^^ ArgumentTypeCoercion: Argument $size of page() expects 'literal-int', got 'int' — coercion may fail at runtime
    page(count($rows));
//       ^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $size of page() expects 'literal-int', got 'int<0, max>' — coercion may fail at runtime
    page($requested + 1);
//       ^^^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $size of page() expects 'literal-int', got 'int' — coercion may fail at runtime
}
