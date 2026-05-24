===description===
undefinedClassForCallable
===file===
<?php
class Foo {
    public function __construct(UndefinedClass $o) {}
}
new Foo(function() : void {});
===expect===
UndefinedClass@3:32: Class UndefinedClass does not exist
UnusedParam@3:32: Parameter $o is never used
