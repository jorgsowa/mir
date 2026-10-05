===description===
A closure param already narrower than the bound type arg keeps its declared type (and is still flagged as too narrow for the callable).
===file===
<?php
interface Animal {}
class Pet implements Animal {}
class Wild {}
enum Err { case NotFound; case Denied; case Timeout; }
class NotFound extends \Exception {}
class Denied extends \Exception {}

/**
 * @template E
 * @template T
 */
class Result {
    /**
     * @template X of \Throwable
     * @param callable(E): X $f
     * @return T
     */
    public function getOrThrow(callable $f) { throw $f(null); }
}

class Plain {
    /**
     * @template X of \Throwable
     * @param callable(Err): X $f
     */
    public function run(callable $f): void { $f(Err::Denied); }
}

/** @return Result<Animal, int> */
function load(): Result { return new Result(); }

load()->getOrThrow(static function (Pet $e): NotFound {
//                 ^ +3:1 ArgumentTypeCoercion: Argument $f of getOrThrow() expects 'callable whose parameter #1 accepts Animal', got 'callable whose parameter #1 only accepts Pet' — coercion may fail at runtime
    /** @mir-check $e is Pet */
    return new NotFound();
});
===expect===
