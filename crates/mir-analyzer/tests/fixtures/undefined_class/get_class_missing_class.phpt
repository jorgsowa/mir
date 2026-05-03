===description===
getClassMissingClass
===file===
<?php
                    class A {}
                    class B {}

                    $a = rand(0, 10) ? new A() : new B();

                    $a = match (get_class($a)) {
                        C::class => 5,
                    };
===expect===
UndefinedClass
===ignore===
TODO
