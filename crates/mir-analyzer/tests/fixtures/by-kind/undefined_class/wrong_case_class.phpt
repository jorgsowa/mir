===description===
Wrong case class name is now reported as WrongCaseClass, not UndefinedClass.
===file===
<?php
class Foo {}
(new foo());
//   ^^^ WrongCaseClass: Class name 'foo' has incorrect casing; use 'Foo'
===expect===
