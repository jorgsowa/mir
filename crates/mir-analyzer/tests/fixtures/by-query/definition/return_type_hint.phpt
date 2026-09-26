===description===
A class in a return type hint lands on the class declaration.
===cursor===
definition
===file===
<?php
final class Greeter {}
function make(): Gree<CURSOR>ter { return new Greeter(); }
===expect===
test.php@2:6-2:22
