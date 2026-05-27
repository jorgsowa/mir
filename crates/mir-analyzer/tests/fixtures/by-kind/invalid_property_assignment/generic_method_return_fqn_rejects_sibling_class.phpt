===description===
Generic method return from different namespace rejects assignment to sibling class in same stub namespace
===file:Prophecy/ObjectProphecy.php===
<?php
namespace Prophecy;

/** @template T */
class ObjectProphecy {}

/** @template T */
class SubjectProphecy {}

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
===file:App/MyTest.php===
<?php
namespace App;

use Prophecy\ObjectProphecy;
use Prophecy\SubjectProphecy;
use Prophecy\Prophet;

class MyTest {
    public SubjectProphecy $prop;

    public function run(): void {
        $prophet = new Prophet();
        $result = $prophet->prophesize(\stdClass::class);
        $this->prop = $result;
    }
}
===expect===
ObjectProphecy.php: UnusedParam@16:32: Parameter $cls is never used
MyTest.php: InvalidPropertyAssignment@14:9: Property $prop expects 'Prophecy\SubjectProphecy', cannot assign 'Prophecy\ObjectProphecy<stdClass>'
