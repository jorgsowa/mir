===description===
Go-to-definition on a `define()`d constant lands on the `define()` call.
===ignore===
===cursor===
definition
===file===
<?php
define('MAXV', 5);
echo MA<CURSOR>XV;
===expect===
test.php@2:0-2:18
