===description===
Get class missing class
===config===
suppress=UnusedVariable
===file===
<?php
class A {}
class B {}

$a = rand(0, 10) ? new A() : new B();

$a = match (get_class($a)) {
    C::class => 5,
};
===expect===
UndefinedClass@8:5-8:6: Class C does not exist
