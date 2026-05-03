===description===
resetAsLazyProxyWithBadType_2
===file===
<?php
                    class Foo {}
                    class Bar {}
                    $reflectionClass = new ReflectionClass(Foo::class);
                    $reflectionClass->resetAsLazyProxy(new Foo, fn(Bar $bar) => new Foo);
===expect===
Argument 2 of ReflectionClass::resetAsLazyProxy expects impure-callable(Foo):Foo, but pure-Closure(Bar):Foo provided
===ignore===
TODO
