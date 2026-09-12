===description===
Null comparisons on nested generic array reads are allowed.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param array<string, array<string, int>> $matrix */
function testOuter(array $matrix): void {
    $row = $matrix['a'];
    if ($row !== null) {}
}

/** @param array<string, array<string, int>> $matrix */
function testInner(array $matrix): void {
    $row = $matrix['a'];
    if ($row === null) {
        return;
    }
    $v = $row['b'];
    if ($v !== null) {}
}
===expect===
