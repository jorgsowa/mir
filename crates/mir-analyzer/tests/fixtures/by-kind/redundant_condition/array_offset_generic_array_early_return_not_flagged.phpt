===description===
Early returns for null generic array reads are allowed.
===config===
suppress=UnusedParam
===file===
<?php
/** @param array<string, array<string, int>> $matrix */
function test(array $matrix): int {
    $row = $matrix['a'];
    if ($row === null) {
        return 0;
    }
    return $row['b'] ?? 0;
}
===expect===
