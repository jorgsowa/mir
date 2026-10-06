===description===
Undefined property fetch
===file===
<?php
class A {
}

echo (new A)->foo;
//            ^^^ UndefinedProperty: Property A::$foo does not exist
