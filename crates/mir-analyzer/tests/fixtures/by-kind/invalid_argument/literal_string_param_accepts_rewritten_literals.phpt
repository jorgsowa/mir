===description===
String functions applied to literal text, and predefined constants, which the
platform fixes, are literal text; the same functions applied to input aren't.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param literal-string $sql */
function query(string $sql): void {}

function run(string $table): void {
    query(substr('SELECT * FROM t', 0, 8));
    query(str_replace('{t}', 'users', 'SELECT * FROM {t}'));
    query(str_pad('id', 10));
    query(ucfirst('select 1'));
    query('a' . DIRECTORY_SEPARATOR . 'b');
    query(PHP_OS_FAMILY);
    query(strtoupper($_GET['order']));
//        ^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $sql of query() expects 'literal-string', got 'uppercase-string'
    query(substr($table, 0, 3));
//        ^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $sql of query() expects 'literal-string', got 'string'
}
