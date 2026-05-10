===description===
undefinedClassForCallable
===file===
<?php
                    class Foo {
                        public function __construct(UndefinedClass $o) {}
                    }
                    new Foo(function() : void {});
===expect===
UndefinedClass@3:52: Class UndefinedClass does not exist
UnusedParam@3:52: Parameter $o is never used
