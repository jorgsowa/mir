===description===
Generic method return type — outer class FQN resolved when defined in a different namespace
===file:Prophecy/Prophecy/ObjectProphecy.php===
<?php
namespace Prophecy\Prophecy;

/** @template T */
class ObjectProphecy {}
===file:Prophecy/Prophecy/Prophet.php===
<?php
namespace Prophecy\Prophecy;

class Prophet {
    /**
     * @template T of object
     * @param class-string<T> $cls
     * @return ObjectProphecy<T>
     */
    public function prophesize(string $cls): ObjectProphecy {
        return new ObjectProphecy();
    }
}
===file:MyApp/Foo.php===
<?php
namespace MyApp;

class Foo {}
===file:MyApp/MyTest.php===
<?php
namespace MyApp;

use Prophecy\Prophecy\ObjectProphecy;
use Prophecy\Prophecy\Prophet;

class MyTest {
    public ObjectProphecy $prophecy;

    public function setUp(): void {
        $prophet = new Prophet();
        $prophecy = $prophet->prophesize(Foo::class);
        /** @mir-check $prophecy is Prophecy\Prophecy\ObjectProphecy<MyApp\Foo> */
        $this->prophecy = $prophecy;
    }
}
===expect===
MyTest.php: MissingConstructor@7:0-7:14: Class MyApp\MyTest has uninitialized properties but no constructor
Prophet.php: UnusedParam@10:32-10:43: Parameter $cls is never used
