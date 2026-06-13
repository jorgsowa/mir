===description===
cross-file: bare generic PHP-typed property accepts parameterized return type (template inference)
===file:Prophecy.php===
<?php
namespace Prophecy\Prophecy;

/** @template T */
class ObjectProphecy {}
===file:TestCase.php===
<?php
namespace PHPUnit\Framework;

use Prophecy\Prophecy\ObjectProphecy;

class TestCase {
    /**
     * @template T of object
     * @param class-string<T> $classOrInterface
     * @return ObjectProphecy<T>
     */
    public function prophesize(string $classOrInterface): ObjectProphecy {
        return new ObjectProphecy();
    }
}
===file:App.php===
<?php
use PHPUnit\Framework\TestCase;
use Prophecy\Prophecy\ObjectProphecy;

class Foo {}

class MyTest extends TestCase {
    public ObjectProphecy $prophecy;

    public function setUp(): void {
        $prophecy = $this->prophesize(Foo::class);
        /** @mir-check $prophecy is Prophecy\Prophecy\ObjectProphecy<Foo> */
        $this->prophecy = $prophecy;
    }
}
===expect===
App.php: MissingConstructor@7:0-7:31: Class MyTest has uninitialized properties but no constructor
TestCase.php: UnusedParam@12:32-12:56: Parameter $classOrInterface is never used
