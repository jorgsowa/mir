===description===
Wrong case class name in new expression is reported.
===file===
<?php
class Foo {}
$x = new foo();
===expect===
WrongCaseClass@3:10-3:13: Class name 'foo' has incorrect casing; use 'Foo'
