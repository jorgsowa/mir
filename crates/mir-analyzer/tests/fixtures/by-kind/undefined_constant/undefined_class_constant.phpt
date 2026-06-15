===description===
Undefined class constant
===file===
<?php
class A {}
echo A::HELLO;
===expect===
UndefinedConstant@3:5-3:13: Constant A::HELLO is not defined
