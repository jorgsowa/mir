===description===
A union type arg narrows to the union; a case outside it is still not required.
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

/** @return Result<Err::NotFound|Err::Denied, int> */
function load(): Result { return new Result(); }

load()->getOrThrow(static fn(Err $e) => match ($e) {
    Err::NotFound => new NotFound(),
    Err::Denied => new Denied(),
});
===expect===
