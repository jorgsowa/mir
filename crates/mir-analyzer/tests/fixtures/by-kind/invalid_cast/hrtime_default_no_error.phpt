===description===
hrtime() with default (false) returns array{0: int, 1: int}|false — no InvalidCast on array access

===config===
suppress=UnusedVariable
===file===
<?php
$t = hrtime();
/** @mir-check $t is array{0: int, 1: int}|false */

===expect===
