===description===
Generic array reads retain their value type when passed as arguments.
===config===
suppress=UnusedParam
===file===
<?php
/** @param array<int, string> $map */
function test(array $map): string {
    return strtoupper($map[5]);
}
===expect===
