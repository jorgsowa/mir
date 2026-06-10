===description===
Undefined property assignment
===ignore===
TODO
===file===
<?php
class A {
}

(new A)->foo = "cool";
===expect===
