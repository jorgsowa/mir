===description===
Conservative: a general string type (not a literal) vs int should not be flagged.
The string could be "0", "123", or any numeric value that would equal an integer.
===config===
php_version=8.0
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(string $s, int $n): void {
    if ($s == $n) {}
}
===expect===
