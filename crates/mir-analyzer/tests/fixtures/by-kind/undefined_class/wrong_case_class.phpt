===description===
Wrong case class name is now reported as WrongCaseClass, not UndefinedClass.
===file===
<?php
class Foo {}
(new foo());
===expect===
WrongCaseClass@3:5-3:8: Class name 'foo' has incorrect casing; use 'Foo'
