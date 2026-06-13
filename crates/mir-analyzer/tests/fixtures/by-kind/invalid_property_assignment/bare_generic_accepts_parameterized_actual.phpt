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
        return new ObjectProphecy();
    }
}

class Foo {}

class MyTest extends TestCase {
    public ObjectProphecy $prophecy;

    public function setUp(): void {
        $prophecy = $this->prophesize(Foo::class);
        /** @mir-check $prophecy is ObjectProphecy<Foo> */
        $this->prophecy = $prophecy;
    }
}
===expect===
UnusedParam@11:32-11:43: Parameter $cls is never used
MissingConstructor@18:0-18:31: Class MyTest has uninitialized properties but no constructor
