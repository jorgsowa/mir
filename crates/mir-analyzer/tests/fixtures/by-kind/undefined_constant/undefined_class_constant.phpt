===description===
Undefined class constant
===file===
<?php
class A {}
echo A::HELLO;
//   ^^^^^^^^ UndefinedConstant: Constant A::HELLO is not defined
===expect===
