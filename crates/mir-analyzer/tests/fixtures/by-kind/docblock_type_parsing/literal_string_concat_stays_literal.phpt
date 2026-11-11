===description===
Inside the function a `literal-string` joined with literals is still a `literal-string`; joined with runtime data it is not.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param literal-string $sql */
function run(string $sql): void {}

/**
 * @param literal-string $table
 * @param literal-int $limit
 */
function select(string $table, int $limit, string $filter): void {
    $base = 'SELECT * FROM ' . $table;
    /** @mir-check $base is non-empty-literal-string */
    run($base);
    $limited = $base . ' LIMIT ' . $limit;
    /** @mir-check $limited is non-empty-literal-string */
    run($limited);
    $filtered = $base . ' WHERE ' . $filter;
    /** @mir-check $filtered is non-empty-string */
    run($filtered);
//      ^^^^^^^^^ ArgumentTypeCoercion: Argument $sql of run() expects 'literal-string', got 'non-empty-string' — coercion may fail at runtime
}
