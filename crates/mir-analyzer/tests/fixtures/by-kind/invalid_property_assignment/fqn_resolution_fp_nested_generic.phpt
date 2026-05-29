===description===
FP guard: nested generic return type Outer<Inner<T>> resolves both classes correctly
===file:Lib/Types.php===
<?php
namespace Lib;

/** @template T */
class Outer {}

/** @template T */
class Inner {}

class Factory {
    /**
     * @template T of object
     * @param class-string<T> $cls
     * @return Outer<Inner<T>>
     */
    public function make(string $cls): Outer {
        return new Outer();
    }
}
===file:App/UseNested.php===
<?php
namespace App;

use Lib\Outer;
use Lib\Factory;

class Foo {}

class Consumer {
    public Outer $prop;

    public function run(): void {
        $factory = new Factory();
        $result = $factory->make(Foo::class);
        /** @mir-check $result is Lib\Outer<Lib\Inner<App\Foo>> */
        $this->prop = $result;
    }
}
===expect===
Types.php: UnusedParam@16:26-16:37: Parameter $cls is never used
