===description===
conditional type with identical branches collapses to constant type
===file===
<?php
/** @template T */
class Container {}

class TestFactory {
    /**
     * @return ($x is null ? Container<object> : Container<object>)
     */
    public function makeContainer($x): Container { return new Container(); }

    public Container $container;
}

$factory = new TestFactory();
$container = $factory->makeContainer(null);
/** @mir-check $container is Container<object> */
$factory->container = $container;
===expect===
MissingConstructor@5:0-5:19: Class TestFactory has uninitialized properties but no constructor
UnusedParam@9:34-9:36: Parameter $x is never used
