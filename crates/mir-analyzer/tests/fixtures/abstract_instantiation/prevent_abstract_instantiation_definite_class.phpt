===description===
preventAbstractInstantiationDefiniteClass
===file===
<?php
                    abstract class A {}

                    function foo(string $a_class) : void {
                        if ($a_class === A::class) {
                            new $a_class();
                        }
                    }
===expect===
AbstractInstantiation@6:32: Cannot instantiate abstract class A
