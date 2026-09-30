===description===
conditional type with identical branches collapses to constant type
===file===
<?php
/** @template T */
class Container {}

class TestFactory {
//<^^^^^^^^^^^^^^^^^^^ MissingConstructor: Class TestFactory has uninitialized properties but no constructor
    /**
     * @return ($x is null ? Container<object> : Container<object>)
     */
    public function makeContainer($x): Container { return new Container(); }
//                                ^^ UnusedParam: Parameter $x is never used

    public Container $container;
}

$factory = new TestFactory();
$container = $factory->makeContainer(null);
/** @mir-check $container is Container<object> */
$factory->container = $container;
===expect===
