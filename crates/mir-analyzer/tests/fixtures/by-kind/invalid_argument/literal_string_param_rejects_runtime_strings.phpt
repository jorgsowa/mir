===description===
A `literal-string` parameter rejects any string that can be read or built at
runtime, including runtime parts joined with literals.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param literal-string $sql */
function query(string $sql): void {}

final class Repo {
    /** @param literal-string $sql */
    public function select(string $sql): void {}
}

/** @param list<string> $columns */
function run(Repo $repo, string $name, int $limit, array $columns): void {
    query("SELECT * FROM users WHERE name = '$name'");
//        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $sql of query() expects 'literal-string', got 'non-empty-string'
    query('SELECT * FROM users LIMIT ' . $limit);
//        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $sql of query() expects 'literal-string', got 'non-falsy-string'
    query(implode(', ', $columns));
//        ^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $sql of query() expects 'literal-string', got 'string'
    query(strtolower($name));
//        ^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $sql of query() expects 'literal-string', got 'lowercase-string'
    $repo->select(sprintf('SELECT * FROM %s', $name));
//                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $sql of select() expects 'literal-string', got 'non-empty-string'
}

/** @param 'ASC'|'DESC' $direction */
function order(string $direction): void {}

function enumLike(string $dir): void {
    order($dir);
//        ^^^^ ArgumentTypeCoercion: Argument $direction of order() expects '"ASC"|"DESC"', got 'string' — coercion may fail at runtime
}
