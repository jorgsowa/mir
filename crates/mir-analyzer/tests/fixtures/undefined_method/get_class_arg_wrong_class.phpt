===description===
getClassArgWrongClass
===file===
<?php
                    class A {}

                    class B {}

                    $a = rand(0, 10) ? new A() : new B();

                    $a = match (get_class($a)) {
                        A::class => $a->barBar(),
                    };
===expect===
UndefinedMethod
===ignore===
TODO
