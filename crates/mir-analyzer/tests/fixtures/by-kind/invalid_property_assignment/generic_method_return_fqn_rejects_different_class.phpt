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
    public ClassB $holder;

    public function run(): void {
        $factory = new Factory();
        $result = $factory->makeA(\stdClass::class);
        $this->holder = $result;
    }
}
===expect===
ClassA.php: UnusedParam@16:27-16:38: Parameter $cls is never used
Consumer.php: MissingConstructor@8:0-8:16: Class App\Consumer has uninitialized properties but no constructor
Consumer.php: InvalidPropertyAssignment@14:9-14:32: Property $holder expects 'Lib\ClassB', cannot assign 'Lib\ClassA<stdClass>'
