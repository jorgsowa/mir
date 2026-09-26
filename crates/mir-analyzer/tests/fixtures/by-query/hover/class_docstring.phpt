===description===
Hover on a class reference shows the class docblock.
===ignore===
===cursor===
hover
===file===
<?php
/** A friendly greeter. */
final class Greeter {}
$g = new Gre<CURSOR>eter();
===expect===
type: Greeter
docstring: A friendly greeter.
definition: test.php@3:6-3:22
