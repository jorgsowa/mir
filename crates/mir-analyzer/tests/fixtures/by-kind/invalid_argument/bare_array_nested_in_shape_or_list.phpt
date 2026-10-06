===description===
A bare `array` nested inside a list, array or shape parameter is accepted like
a top-level one (a coercion, not an error).
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function untyped(): array { return []; }

/** @param list<array{id: non-empty-string}> $rows */
function takeList(array $rows): void {}

/** @param array<array{id: int}> $rows */
function takeArray(array $rows): void {}

/** @param array{item: array{id: int}} $shape */
function takeShape(array $shape): void {}

/** @param array{id: int} $row */
function takeRow(array $row): void {}

/** @return list<array> */
function rows(): array { return []; }

/** @return array<string, array> */
function keyedRows(): array { return []; }

/** @mir-check untyped() is array<array-key, mixed> */
takeList([untyped()]);
takeList([untyped(), untyped()]);
takeList(rows());
takeArray(rows());
takeArray(keyedRows());
takeShape(['item' => untyped()]);
takeRow(untyped());
