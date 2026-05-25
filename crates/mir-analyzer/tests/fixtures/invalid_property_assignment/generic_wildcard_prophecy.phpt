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
}

class Container {
    public GenericWrapper $bare;
}

$factory = new WrapperFactory();
$c = new Container();
$wrapper = $factory->make(stdClass::class);
/** @mir-check $wrapper is GenericWrapper<object> */
$c->bare = $wrapper;
===expect===
UnusedParam@11:26: Parameter $cls is never used
