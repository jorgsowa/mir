===description===
Undefined property assignment
===file===
<?php
class A {
}

(new A)->foo = "cool";
===expect===
