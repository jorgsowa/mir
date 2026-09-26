===description===
Class names are case-insensitive, so a differently-cased `new` still resolves.
===cursor===
definition
===file===
<?php
final class Greeter {}
$g = new gree<CURSOR>ter();
===expect===
test.php@2:6-2:22
