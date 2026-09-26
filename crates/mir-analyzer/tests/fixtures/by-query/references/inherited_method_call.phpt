===description===
A call through a subclass counts as a reference to the declaring class's method.
===cursor===
references include_declaration
===file===
<?php
class Base { public function greet(): void {} }
final class Child extends Base {}
(new Base())->gr<CURSOR>eet();
(new Child())->greet();
===expect===
test.php@2:29-2:34
test.php@4:14-4:19
test.php@5:15-5:20
