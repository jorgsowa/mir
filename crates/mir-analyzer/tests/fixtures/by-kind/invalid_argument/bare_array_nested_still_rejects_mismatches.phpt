===description===
Nested arrays that cannot hold the expected shape are still rejected: a
wrongly typed or wrongly keyed array, and a scalar where a shape is expected.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param list<array{id: int}> $rows */
function takeList(array $rows): void {}

/** @return list<array<string, string>> */
function stringRows(): array { return []; }

/** @return list<array<int, int>> */
function intKeyedRows(): array { return []; }

takeList(stringRows());
//       ^^^^^^^^^^^^ InvalidArgument: Argument $rows of takeList() expects 'list<array{'id': int}>', got 'list<array<string, string>>'
takeList(intKeyedRows());
//       ^^^^^^^^^^^^^^ InvalidArgument: Argument $rows of takeList() expects 'list<array{'id': int}>', got 'list<array<int, int>>'
takeList([1]);
//       ^^^ InvalidArgument: Argument $rows of takeList() expects 'list<array{'id': int}>', got 'array{0: 1}'
===expect===
