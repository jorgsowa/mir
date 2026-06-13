===description===
count() over a shape with a required key is int<1, max>
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param array{id: int, name?: string} $row */
function test(array $row): void {
    $n = count($row);
    /** @mir-check $n is int<1, max> */
    $_ = $n;
}
===expect===
