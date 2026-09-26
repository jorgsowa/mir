===description===
Method references match the declaring class, not every method sharing the name.
===cursor===
references
===file===
<?php
final class A { public function run(): void {} }
final class B { public function run(): void {} }
(new A())->r<CURSOR>un();
(new B())->run();
(new A())->run();
===expect===
test.php@4:11-4:14
test.php@6:11-6:14
