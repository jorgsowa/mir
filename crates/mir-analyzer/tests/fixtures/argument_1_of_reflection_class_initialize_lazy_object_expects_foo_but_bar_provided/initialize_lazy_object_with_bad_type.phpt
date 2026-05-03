===description===
initializeLazyObjectWithBadType
===file===
<?php
                    class Foo {}
                    class Bar {}
                    $reflectionClass = new ReflectionClass(Foo::class);
                    $reflectionClass->initializeLazyObject(new Bar);
===expect===
Argument 1 of ReflectionClass::initializeLazyObject expects Foo, but Bar provided
===ignore===
TODO
