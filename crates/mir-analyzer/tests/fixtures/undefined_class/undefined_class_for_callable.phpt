===description===
Undefined class for callable
===file===
<?php
class Foo {
    public function __construct(UndefinedClass $o) {}
}
new Foo(function() : void {});
===expect===
UndefinedClass@3:33: Class UndefinedClass does not exist
UnusedParam@3:33: Parameter $o is never used
