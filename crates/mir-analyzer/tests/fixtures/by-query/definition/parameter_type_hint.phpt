===description===
A class in a parameter type hint lands on the class declaration.
===cursor===
definition
===file===
<?php
final class Greeter {}
function f(Gree<CURSOR>ter $g): void {}
===expect===
test.php@2:6-2:22
