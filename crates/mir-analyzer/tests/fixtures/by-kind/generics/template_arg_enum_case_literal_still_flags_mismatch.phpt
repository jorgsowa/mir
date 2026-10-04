===description===
An inferred enum-case binding is still compared exactly: another case, another enum and a
partial case union are all reported.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
enum Level { case Low; case High; }
enum Mode { case Fast; }
enum Tier { case One; case Two; case Three; }

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
}

/** @return Outcome<Level::Low, int> */
function otherCase(): Outcome { return Outcome::failure(Level::High); }

/** @return Outcome<Level::Low, int> */
function otherEnum(): Outcome { return Outcome::failure(Mode::Fast); }

/** @return Outcome<Tier::One|Tier::Two, int> */
function partialUnion(): Outcome { return Outcome::failure(Tier::Three); }

/** @param Outcome<Level::Low, int> $o */
function take(Outcome $o): void {}

function passWrongCase(): void {
    take(Outcome::failure(Level::High));
}
===expect===
InvalidReturnType@20:32-20:69: Return type 'Outcome<Level::High, never>' is not compatible with declared 'Outcome<Level::Low, int>'
InvalidReturnType@23:32-23:68: Return type 'Outcome<Mode::Fast, never>' is not compatible with declared 'Outcome<Level::Low, int>'
InvalidReturnType@26:35-26:72: Return type 'Outcome<Tier::Three, never>' is not compatible with declared 'Outcome<Tier::One|Tier::Two, int>'
InvalidArgument@32:9-32:38: Argument $o of take() expects 'Outcome<Level::Low, int>', got 'Outcome<Level::High, never>'
