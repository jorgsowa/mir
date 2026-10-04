===description===
A declared enum-case literal in a template-argument slot accepts the enum-typed value
(`E::Case` expressions are typed as the bare enum), whether the generic comes from a declared
return, an inferred template binding, or a call argument. `never` fits any invariant slot. Covariant slots behave the same.
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
 * @template E
 * @template T
 */
final class Outcome {
    /** @return Outcome<Level, never> */
    public static function declared(Level $l): self { return new self(); }

    /**
     * @template X
     * @param X $e
     * @return Outcome<X, never>
     */
    public static function inferred(mixed $e): self { return new self(); }

    /**
     * @template X
     * @param X $v
     * @return Outcome<never, X>
     */
    public static function value(mixed $v): self { return new self(); }
}

/** @return Outcome<Level::Low, int> */
function fromDeclared(): Outcome {
    $r = Outcome::declared(Level::Low);
    /** @mir-check $r is Outcome<Level, never> */
    return $r;
}

/** @return Outcome<Level::Low, int> */
function fromInferred(): Outcome {
    $r = Outcome::inferred(Level::Low);
    /** @mir-check $r is Outcome<Level, never> */
    return $r;
}

/** @return Outcome<Level::Low|Level::High, never> */
function unionOfCases(): Outcome { return Outcome::inferred(Level::Low); }

/** @return Outcome<never, Level::High> */
function neverSlot(): Outcome { return Outcome::value(Level::High); }

/** @param Outcome<Level::Low, int> $o */
function take(Outcome $o): void {}

function passArg(): void {
    take(Outcome::declared(Level::Low));
    take(Outcome::inferred(Level::High));
}
/**
 * @template-covariant E
 * @template-covariant T
 */
final class CoOutcome {
    /**
     * @template X
     * @param X $e
     * @return CoOutcome<X, never>
     */
    public static function inferred(mixed $e): self { return new self(); }
}

/** @return CoOutcome<Level::Low, int> */
function covariantSlot(): CoOutcome { return CoOutcome::inferred(Level::Low); }

/** @param CoOutcome<Level::Low, int> $o */
function takeCo(CoOutcome $o): void {}

function passCo(): void {
    takeCo(CoOutcome::inferred(Level::High));
}
===expect===
