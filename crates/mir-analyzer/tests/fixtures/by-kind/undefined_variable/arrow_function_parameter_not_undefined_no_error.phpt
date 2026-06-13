===description===
arrow function parameter not undefined no error
===config===
suppress=UnusedVariable
===file===
<?php
$fn = fn(int $n): int => $n * 2;
===expect===
