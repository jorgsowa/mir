===description===
A callable signature that passes a parameter by reference needs a callback taking it by reference.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param callable(array<string, int> &$row): void $update */
function eachRow(callable $update): void {}

eachRow(function (array &$row): void {
    $row['seen'] = 1;
});
eachRow(function (array $row): void {
//      ^ +2:1 InvalidArgument: Argument $update of eachRow() expects 'callable taking parameter #1 by reference', got 'callable taking parameter #1 by value'
    $row['seen'] = 1;
});
eachRow(static fn (array $row) => null);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $update of eachRow() expects 'callable taking parameter #1 by reference', got 'callable taking parameter #1 by value'
eachRow(static fn () => null);
