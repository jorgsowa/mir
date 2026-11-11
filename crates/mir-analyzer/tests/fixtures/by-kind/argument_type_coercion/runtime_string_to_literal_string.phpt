===description===
A string read at runtime is a coercion to `literal-string`; literals, constants, class names and their concatenations are accepted.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Schema {
    public const TABLE = 'users';
}

/** @param literal-string $sql */
function query(string $sql): void {}

function run(string $input, bool $all): void {
    query('SELECT 1');
    query('SELECT * FROM ' . Schema::TABLE);
    query(Schema::TABLE);
    query(Schema::class);
    query($all ? 'SELECT *' : 'SELECT id');
    query('');
    query($input);
//        ^^^^^^ ArgumentTypeCoercion: Argument $sql of query() expects 'literal-string', got 'string' — coercion may fail at runtime
    query('SELECT * FROM ' . $input);
//        ^^^^^^^^^^^^^^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $sql of query() expects 'literal-string', got 'non-empty-string' — coercion may fail at runtime
    query("SELECT $input");
//        ^^^^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $sql of query() expects 'literal-string', got 'non-empty-string' — coercion may fail at runtime
    query(trim($input));
//        ^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $sql of query() expects 'literal-string', got 'string' — coercion may fail at runtime
}
