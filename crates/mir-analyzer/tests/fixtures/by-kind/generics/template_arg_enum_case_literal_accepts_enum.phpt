===description===
`Enum::Case` passed to a template parameter binds the case itself, so the inferred generic fits a
declared single-case type argument. Works for methods, constructors, functions, covariant and
invariant templates, a case union, and `never` slots; a bare enum stays usable for `Box<Enum>`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
enum Level { case Low; case High; }

/**
 * @template-covariant E
 * @template-covariant T
 */
final class Outcome {
    /**
     * @template X
     * @param X $e
     * @return Outcome<X, never>
     */
    public static function failure(mixed $e): self { return new self(); }

    /**
     * @template X
     * @param X $v
     * @return Outcome<never, X>
     */
    public static function value(mixed $v): self { return new self(); }
}

/**
 * @template E
 * @template T
 */
final class Exact {
    /**
     * @template X
     * @param X $e
     * @return Exact<X, never>
     */
    public static function failure(mixed $e): self { return new self(); }
}

/** @template T */
final class Box {
    /** @param T $v */
    public function __construct(public $v) {}
}

/**
 * @template X
 * @param X $e
 * @return Outcome<X, never>
 */
function fail(mixed $e): Outcome { return new Outcome(); }

/** @return Outcome<Level::Low, int> */
function viaMethod(): Outcome {
    $r = Outcome::failure(Level::Low);
    /** @mir-check $r is Outcome<Level::Low, never> */
    return $r;
}

/** @return Outcome<Level::Low, int> */
function viaFunction(): Outcome { return fail(Level::Low); }

/** @return Outcome<Level::Low|Level::High, never> */
function caseUnion(): Outcome { return Outcome::failure(Level::Low); }

/** @return Outcome<never, Level::High> */
function neverSlot(): Outcome { return Outcome::value(Level::High); }

/** @return Exact<Level::Low, int> */
function invariantExact(): Exact { return Exact::failure(Level::Low); }

/** @return Box<Level> */
function boxOfBareEnum(): Box { return new Box(Level::Low); }

/** @param Outcome<Level::Low, int> $o */
function take(Outcome $o): void {}

function passArg(): void {
    take(Outcome::failure(Level::Low));
}
===expect===
