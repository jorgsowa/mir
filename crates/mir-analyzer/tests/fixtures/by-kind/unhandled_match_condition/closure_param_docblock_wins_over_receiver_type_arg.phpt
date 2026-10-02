===description===
An explicit docblock @param on the closure is not overridden by the callable signature.
===file===
<?php
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

/** @return Result<Err::NotFound, int> */
function load(): Result { return new Result(); }

load()->getOrThrow(
    /** @param Err $e */
    static function (Err $e): NotFound {
        /** @mir-check $e is Err */
        return new NotFound();
    },
);
===expect===
