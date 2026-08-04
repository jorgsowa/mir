===description===
L3, nested generic arrays: both the outer and the inner offset read are
individually uncertain (`array<string, array<string,int>>`), so a null
check on either level's read must not be flagged.
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
