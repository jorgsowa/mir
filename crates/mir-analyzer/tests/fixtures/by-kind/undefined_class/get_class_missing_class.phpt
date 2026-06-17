===description===
Foo::class in a match arm does not emit UndefinedClass — ::class is a compile-time
string constant that does not require the class to be defined.
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
