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
//                             ^^^^^^^^^^^ UnusedParam: Parameter $cls is never used
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
//<^^^^^^^^^^^^^^ MissingConstructor: Class App\MyTest has uninitialized properties but no constructor
    public SubjectProphecy $prop;

    public function run(): void {
        $prophet = new Prophet();
        $result = $prophet->prophesize(\stdClass::class);
        $this->prop = $result;
//      ^^^^^^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $prop expects 'Prophecy\SubjectProphecy', cannot assign 'Prophecy\ObjectProphecy<stdClass>'
    }
}
===expect===
