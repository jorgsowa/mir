===description===
Undefined class for callable
===config===
suppress=UnusedParam,UnusedFunction
===file===
<?php
class Foo {
    public function __construct(UndefinedClass $o) {}
}
new Foo(function() : void {});
===expect===
UndefinedClass@3:33-3:47: Class UndefinedClass does not exist
