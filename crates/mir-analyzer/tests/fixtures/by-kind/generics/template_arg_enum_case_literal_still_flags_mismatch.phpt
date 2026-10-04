===description===
Single-case acceptance in template-argument slots does not hide a different enum, a
specific other case, or a non-enum mismatch.
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
 * @template E
 * @template T
 */
final class Outcome {
    /** @return Outcome<Level::High, int> */
    public static function high(): self { return new self(); }

    /** @return Outcome<Mode, int> */
    public static function mode(): self { return new self(); }

    /** @return Outcome<Tier, int> */
    public static function tier(): self { return new self(); }

    /** @return Outcome<string, int> */
    public static function text(): self { return new self(); }
}

/** @return Outcome<Level::Low, int> */
function otherCase(): Outcome { return Outcome::high(); }

/** @return Outcome<Level::Low, int> */
function otherEnum(): Outcome { return Outcome::mode(); }

/** @return Outcome<Level::Low, int> */
function nonEnum(): Outcome { return Outcome::text(); }

/** @return Outcome<Tier::One|Tier::Two, int> */
function partialUnion(): Outcome { return Outcome::tier(); }

/** @param Outcome<Level::Low, int> $o */
function take(Outcome $o): void {}

function passMode(): void {
    take(Outcome::mode());
}
===expect===
InvalidReturnType@25:32-25:55: Return type 'Outcome<Level::High, int>' is not compatible with declared 'Outcome<Level::Low, int>'
InvalidReturnType@28:32-28:55: Return type 'Outcome<Mode, int>' is not compatible with declared 'Outcome<Level::Low, int>'
InvalidReturnType@31:30-31:53: Return type 'Outcome<string, int>' is not compatible with declared 'Outcome<Level::Low, int>'
InvalidReturnType@34:35-34:58: Return type 'Outcome<Tier, int>' is not compatible with declared 'Outcome<Tier::One|Tier::Two, int>'
InvalidArgument@40:9-40:24: Argument $o of take() expects 'Outcome<Level::Low, int>', got 'Outcome<Mode, int>'
