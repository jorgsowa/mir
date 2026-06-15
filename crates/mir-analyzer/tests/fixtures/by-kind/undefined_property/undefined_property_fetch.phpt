===description===
Undefined property fetch
===file===
<?php
class A {
}

echo (new A)->foo;
===expect===
UndefinedProperty@5:14-5:17: Property A::$foo does not exist
