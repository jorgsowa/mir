===description===
A union of every case of an enum is the enum itself inside an invariant type argument, in both directions and across arg and return checks.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnnecessaryVarAnnotation errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
enum Suit { case A; case B; case C; }

/** @template T */
final class Box {
    /** @param T $v */
    public function __construct(public $v) {}
}

/** @param Box<Suit> $b */
function takeEnum(Box $b): void {}

/** @param Box<Suit::A|Suit::B|Suit::C> $b */
function takeCases(Box $b): void {}

/** @return Box<Suit> */
function returnEnum(): Box {
    /** @var Box<Suit::C|Suit::B|Suit::A> $b */
    $b = new Box(Suit::A);
    /** @mir-check $b is Box<Suit::C|Suit::B|Suit::A> */
    return $b;
}

function check(): void {
    /** @var Box<Suit::A|Suit::B|Suit::C> $cases */
    $cases = new Box(Suit::A);
    takeEnum($cases);
    /** @var Box<Suit> $enum */
    $enum = new Box(Suit::A);
    takeCases($enum);
    /** @mir-check $enum is Box<Suit> */
    echo 1;
}
