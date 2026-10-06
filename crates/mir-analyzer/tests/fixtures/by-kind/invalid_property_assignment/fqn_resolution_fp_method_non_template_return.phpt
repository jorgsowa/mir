===description===
FP guard: method with @template but returning a non-template same-namespace class must still be accepted
===file:Lib/Types.php===
<?php
namespace Lib;

class Result {}

/** @template T */
class Box {
    /**
     * @template U of object
     * @param class-string<U> $cls
     * @return Result
     */
    public function info(string $cls): Result {
//                       ^^^^^^^^^^^ UnusedParam: Parameter $cls is never used
        return new Result();
    }
}
===file:App/UseIt.php===
<?php
namespace App;

use Lib\Result;
use Lib\Box;

class Consumer {
//<^^^^^^^^^^^^^^^^ MissingConstructor: Class App\Consumer has uninitialized properties but no constructor
    public Result $result;

    public function run(): void {
        $box = new Box();
        $this->result = $box->info(\stdClass::class);
    }
}
