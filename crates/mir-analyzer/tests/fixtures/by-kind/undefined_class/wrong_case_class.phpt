===description===
Wrong case class name is now reported as WrongCaseClass, not UndefinedClass.
===file===
<?php
class Foo {}
(new foo());
===expect===
WrongCaseClass@3:6-3:9: Class name 'foo' has incorrect casing; use 'Foo'
