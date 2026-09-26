===description===
The class token of `Foo::class` lands on the class declaration.
===cursor===
definition
===file===
<?php
final class Greeter {}
$c = Gree<CURSOR>ter::class;
===expect===
test.php@2:6-2:22
