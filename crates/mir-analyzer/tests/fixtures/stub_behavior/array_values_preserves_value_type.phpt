===description===
array_values re-indexes an array and preserves the element type as a list
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param array<string, int> $map */
function test(array $map): void {
    $vals = array_values($map);
    foreach ($vals as $v) {
        /** @mir-check $v is int */
        $_ = $v;
    }
}

/** @param list<string> $items */
function testList(array $items): void {
    $reindexed = array_values($items);
    foreach ($reindexed as $item) {
        /** @mir-check $item is string */
        $_ = $item;
    }
}
===expect===
