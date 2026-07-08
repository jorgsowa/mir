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
UnhandledMatchCondition@8:5-10:1: Unhandled match condition: possibly-unmatched value of type 'string'
UndefinedMethod@9:16-9:28: Method A::barBar() does not exist
