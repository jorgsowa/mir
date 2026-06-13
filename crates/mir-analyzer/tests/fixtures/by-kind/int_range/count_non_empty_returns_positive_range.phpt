===description===
count() over a statically non-empty collection is int<1, max>
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param non-empty-list<int> $items */
function test(array $items): void {
    $n = count($items);
    /** @mir-check $n is int<1, max> */
    $_ = $n;
}
===expect===
