===description===
Get class arg wrong class
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
class A {}

class B {}

$a = rand(0, 10) ? new A() : new B();

$a = match (get_class($a)) {
    A::class => $a->barBar(),
};
===expect===
UndefinedMethod@9:17-9:29: Method A::barBar() does not exist
