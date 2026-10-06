===description===
A nested index write through a dynamic key keeps the surrounding keyed-shape levels and builds a generic array at the dynamic level.
===config===
<mir>
  <issueHandlers>
    <MissingParamType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Item {}

/** @return array<int, array{open: array<int, Item>, done: array<int, Item>}> */
function dynamicOuterLiteralDynamic(int $row, int $id): array {
    $r = [];
    $r[$row] = ['open' => [], 'done' => []];
    $r[$row]['open'][$id] = new Item();
    return $r;
}

/** @return array{open: array<int, Item>, done: array<int, array<int, Item>>} */
function literalOuterTwoDynamic(int $a, int $b): array {
    $x = ['open' => [], 'done' => []];
    $x['done'][$a][$b] = new Item();
    return $x;
}

function dynamicLeafUnderLiteralPath(int $row, int $id): void {
    $r = [];
    $r[$row] = ['open' => [], 'done' => []];
    $r[$row]['open'][$id] = new Item();
    /** @mir-check $r is array<int, array{open: array<int, Item>, done: array{}}> */
    $r;
}

function literalThenTwoDynamic(int $a, int $b): void {
    $x = ['open' => [], 'done' => []];
    $x['done'][$a][$b] = new Item();
    /** @mir-check $x is array{open: array{}, done: array<int, array<int, Item>>} */
    $x;
}

function literalThenDynamicThenPush(int $a): void {
    $x = ['open' => [], 'done' => []];
    $x['done'][$a][] = new Item();
    /** @mir-check $x is array{open: array{}, done: array<int, non-empty-list<Item>>} */
    $x;
}

function stringKeyedDynamic(string $k, int $id): void {
    $x = ['open' => []];
    $x['open'][$k][$id] = new Item();
    /** @mir-check $x is array{open: array<string, array<int, Item>>} */
    $x;
}

/** @param list<int> $ids */
function dynamicWriteInLoop(array $ids): void {
    $x = ['open' => [], 'done' => []];
    foreach ($ids as $id) {
        $x['open'][$id][$id] = new Item();
    }
    /** @mir-check $x is array{open: array<int, array<int, Item>>, done: array{}}|array{open: array{}, done: array{}} */
    $x;
}

function literalPathStillPrecise(int $row): void {
    $r = [];
    $r[$row] = ['tags' => [], 'name' => 'n'];
    $r[$row]['tags'][] = 't';
    /** @mir-check $r is array<int, array{tags: list<'t'>, name: 'n'}> */
    $r;
}
