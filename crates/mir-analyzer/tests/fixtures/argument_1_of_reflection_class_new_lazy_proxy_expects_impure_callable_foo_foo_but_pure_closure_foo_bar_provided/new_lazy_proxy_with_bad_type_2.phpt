===description===
newLazyProxyWithBadType_2
===file===
<?php
                    class Foo {}
                    class Bar {}
                    $reflectionClass = new ReflectionClass(Foo::class);
                    $reflectionClass->newLazyProxy(fn(Foo $foo) => new Bar);
===expect===
Argument 1 of ReflectionClass::newLazyProxy expects impure-callable(Foo):Foo, but pure-Closure(Foo):Bar provided
===ignore===
TODO
