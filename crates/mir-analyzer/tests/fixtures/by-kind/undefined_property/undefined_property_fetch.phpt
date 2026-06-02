===description===
Undefined property fetch
===file===
<?php
class A {
}

echo (new A)->foo;
===expect===
UndefinedProperty@5:15-5:18: Property A::$foo does not exist
