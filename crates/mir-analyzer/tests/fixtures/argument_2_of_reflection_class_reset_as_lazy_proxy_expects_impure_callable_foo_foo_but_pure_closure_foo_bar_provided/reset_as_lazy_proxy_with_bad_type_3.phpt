===description===
resetAsLazyProxyWithBadType_3
===file===
<?php
                    class Foo {}
                    class Bar {}
                    $reflectionClass = new ReflectionClass(Foo::class);
                    $reflectionClass->resetAsLazyProxy(new Foo, fn(Foo $foo) => new Bar);
===expect===
Argument 2 of ReflectionClass::resetAsLazyProxy expects impure-callable(Foo):Foo, but pure-Closure(Foo):Bar provided
===ignore===
TODO
