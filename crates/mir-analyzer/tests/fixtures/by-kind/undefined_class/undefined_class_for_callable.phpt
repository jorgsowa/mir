===description===
Undefined class for callable
===config===
suppress=UnusedParam,UnusedFunction
===file===
<?php
class Foo {
    public function __construct(UndefinedClass $o) {}
//                              ^^^^^^^^^^^^^^ UndefinedClass: Class UndefinedClass does not exist
}
new Foo(function() : void {});
===expect===
