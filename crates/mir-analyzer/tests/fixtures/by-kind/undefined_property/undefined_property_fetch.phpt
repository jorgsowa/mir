===description===
Undefined property fetch
===file===
<?php
class A {
}

echo (new A)->foo;
===expect===
UndefinedPropertyFetch
===ignore===
TODO
