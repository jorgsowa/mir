===description===
bare PHP-typed property accepts parameterized value from factory method (same FQCN)
===file===
<?php
/** @template T */
class GenericWrapper {}

class WrapperFactory {
    /**
     * @template T of object
     * @param class-string<T> $cls
     * @return GenericWrapper<T>
     */
    public function make(string $cls): GenericWrapper { return new GenericWrapper(); }
//                       ^^^^^^^^^^^ UnusedParam: Parameter $cls is never used
}

class Container {
//<^^^^^^^^^^^^^^^^^ MissingConstructor: Class Container has uninitialized properties but no constructor
    public GenericWrapper $bare;
}

$factory = new WrapperFactory();
$c = new Container();
$wrapper = $factory->make(stdClass::class);
/** @mir-check $wrapper is GenericWrapper<stdClass> */
$c->bare = $wrapper;
===expect===
