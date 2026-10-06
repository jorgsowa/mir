===description===
array_column over rows of a class reads the declared types of its public properties; non-public,
undeclared, templated and union row types fall back to the generic result.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MissingConstructor errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Row {
    /** @var positive-int */
    public int $id;
    /** @var positive-int */
    public int $targetId;
    public string $name;
    private string $secret = '';
    public $untyped;
}

class Other {
    public int $id;
}

/** @template T */
class Box {
    /** @var T */
    public $value;
}

/**
 * @param list<Row> $rows
 * @param list<Row|Other> $mixed_rows
 * @param list<Box<int>> $boxes
 */
function test(array $rows, array $mixed_rows, array $boxes): void {
    $map = array_column($rows, 'targetId', 'id');
    /** @mir-check $map is array<positive-int, positive-int> */
    $_ = $map;

    $names = array_column($rows, 'name');
    /** @mir-check $names is list<string> */
    $_ = $names;

    $objects = array_column($rows, null, 'id');
    /** @mir-check $objects is array<positive-int, Row> */
    $_ = $objects;

    $private = array_column($rows, 'secret');
    /** @mir-check $private is list<mixed> */
    $_ = $private;

    $missing = array_column($rows, 'nope', 'id');
    /** @mir-check $missing is array<array-key, mixed> */
    $_ = $missing;

    $union = array_column($mixed_rows, 'id', 'id');
    /** @mir-check $union is array<array-key, mixed> */
    $_ = $union;

    $templated = array_column($boxes, 'value', 'value');
    /** @mir-check $templated is array<array-key, mixed> */
    $_ = $templated;
}
