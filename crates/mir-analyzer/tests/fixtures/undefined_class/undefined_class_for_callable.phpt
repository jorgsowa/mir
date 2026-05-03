===description===
undefinedClassForCallable
===file===
<?php
                    class Foo {
                        public function __construct(UndefinedClass $o) {}
                    }
                    new Foo(function() : void {});
===expect===
UndefinedClass
===ignore===
TODO
