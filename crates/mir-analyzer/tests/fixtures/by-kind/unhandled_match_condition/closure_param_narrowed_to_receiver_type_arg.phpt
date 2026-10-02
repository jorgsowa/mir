===description===
An arrow fn param declared with a wider native type is narrowed to the receiver's bound error type, so the match needs no arm for the other cases.
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

load()->getOrThrow(static fn(Err $e) => match ($e) {
    Err::NotFound => new NotFound(),
});
===expect===
