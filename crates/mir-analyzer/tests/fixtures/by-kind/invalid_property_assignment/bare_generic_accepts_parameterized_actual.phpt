===description===
bare generic PHP-typed property accepts parameterized actual from method return (template inference exercised)
===file===
<?php
/** @template T */
class ObjectProphecy {}

class TestCase {
    /**
     * @template T of object
     * @param class-string<T> $cls
     * @return ObjectProphecy<T>
     */
    public function prophesize(string $cls): ObjectProphecy {
//                             ^^^^^^^^^^^ UnusedParam: Parameter $cls is never used
        return new ObjectProphecy();
    }
}

class Foo {}

class MyTest extends TestCase {
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MissingConstructor: Class MyTest has uninitialized properties but no constructor
    public ObjectProphecy $prophecy;

    public function setUp(): void {
        $prophecy = $this->prophesize(Foo::class);
        /** @mir-check $prophecy is ObjectProphecy<Foo> */
        $this->prophecy = $prophecy;
    }
}
