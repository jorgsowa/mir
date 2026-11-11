===description===
Matching shapes nested in a list are accepted, including extra keys, int for float and open shapes.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param list<array{id: int, price: float, tag?: string}> $items */
function saveItems(array $items): void {}

/**
 * @param array{id: int, price: float} $row
 * @param array{id: int, ...} $open
 */
function run(array $row, array $open): void {
    saveItems([['id' => 1, 'price' => 2.5]]);
    saveItems([['id' => 1, 'price' => 2]]);
    saveItems([['id' => 1, 'price' => 2.5, 'tag' => 'new']]);
    saveItems([['id' => 1, 'price' => 2.5, 'extra' => true]]);
    saveItems([$row, $row]);
    saveItems([$open]);
}
