===description===
Wrong case class name in new expression is reported.
===config===
suppress=UnusedVariable
===file===
<?php
class Foo {}
$x = new foo();
//       ^^^ WrongCaseClass: Class name 'foo' has incorrect casing; use 'Foo'
===expect===
