===description===
The parent class in an `extends` clause lands on its declaration.
===cursor===
definition
===file===
<?php
class Base {}
final class Child extends Ba<CURSOR>se {}
===expect===
test.php@2:0-2:13
