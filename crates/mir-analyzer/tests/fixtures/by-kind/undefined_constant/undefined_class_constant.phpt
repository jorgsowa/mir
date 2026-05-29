===description===
Undefined class constant
===file===
<?php
class A {}
echo A::HELLO;
===expect===
UndefinedConstant@3:6-3:14: Constant A::HELLO is not defined
