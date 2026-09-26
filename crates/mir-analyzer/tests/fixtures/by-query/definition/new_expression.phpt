===description===
Go-to-definition on the class name in a `new` expression lands on the class declaration.
===cursor===
definition
===file===
<?php
final class Greeter {}
$g = new Gree<CURSOR>ter();
===expect===
test.php@2:6-2:22
