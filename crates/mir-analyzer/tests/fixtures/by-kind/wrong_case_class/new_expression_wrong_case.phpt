===description===
Wrong case class name in new expression is reported.
===config===
suppress=UnusedVariable
===file===
<?php
class Foo {}
$x = new foo();
===expect===
WrongCaseClass@3:9-3:12: Class name 'foo' has incorrect casing; use 'Foo'
