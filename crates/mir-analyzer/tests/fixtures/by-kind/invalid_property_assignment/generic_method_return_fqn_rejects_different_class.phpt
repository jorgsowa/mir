===description===
Generic method return from different namespace still rejects assignment to property of different class
===file:Lib/ClassA.php===
<?php
namespace Lib;

/** @template T */
class ClassA {}

/** @template T */
class ClassB {}

class Factory {
    /**
     * @template T of object
     * @param class-string<T> $cls
     * @return ClassA<T>
     */
    public function makeA(string $cls): ClassA {
//                        ^^^^^^^^^^^ UnusedParam: Parameter $cls is never used
        return new ClassA();
    }
}
===file:App/Consumer.php===
<?php
namespace App;

use Lib\ClassA;
use Lib\ClassB;
use Lib\Factory;

class Consumer {
//<^^^^^^^^^^^^^^^^ MissingConstructor: Class App\Consumer has uninitialized properties but no constructor
    public ClassB $holder;

    public function run(): void {
        $factory = new Factory();
        $result = $factory->makeA(\stdClass::class);
        $this->holder = $result;
//      ^^^^^^^^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $holder expects 'Lib\ClassB', cannot assign 'Lib\ClassA<stdClass>'
    }
}
===expect===
