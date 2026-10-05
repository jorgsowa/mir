===description===
A non-empty array fits a shape param whose keys are all optional; a wrong element type still fails.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array{a?: int, b?: int} $v */
function optionalShape(array $v): void {}

/** @param array{a: int, b?: int} $v */
function requiredShape(array $v): void {}

/** @param list<array{a?: int}> $v */
function nestedShapes(array $v): void {}

/** @param array{a?: int}|null $v */
function nullableShape(?array $v): void {}

/**
 * @param array<string, int> $ints
 * @param array<string, string> $strings
 */
function run(array $ints, array $strings): void {
    optionalShape($ints);
    if ($ints === []) { return; }
    /** @mir-check $ints is non-empty-array<string, int> */
    optionalShape($ints);
    optionalShape($ints);
    requiredShape($ints);
    nullableShape($ints);

    if ($strings === []) { return; }
    /** @mir-check $strings is non-empty-array<string, string> */
    optionalShape($strings);
//                ^^^^^^^^ InvalidArgument: Argument $v of optionalShape() expects 'array{'a'?: int, 'b'?: int}', got 'non-empty-array<string, string>'
}

/** @param non-empty-list<int> $list */
function listToShape(array $list): void {
    /** @param array{0?: int} $v */
    $f = static function (array $v): void {};
    $f($list);
}

/** @param non-empty-array<string, array<string, int>> $rows */
function nested(array $rows): void {
    foreach ($rows as $row) {
        optionalShape($row);
    }
}
===expect===
