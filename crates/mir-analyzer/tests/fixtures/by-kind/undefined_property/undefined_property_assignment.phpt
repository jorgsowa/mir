===description===
Undefined property assignment
===file===
<?php
class A {
}

$a = new A();
$a->foo = "cool";
//<^^^^^^^^^^^^^^^^ DynamicPropertyCreation: Creation of dynamic property A::$foo is deprecated
