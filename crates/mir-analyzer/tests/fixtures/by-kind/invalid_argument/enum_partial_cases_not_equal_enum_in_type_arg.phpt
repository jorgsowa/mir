===description===
A union of only some cases is not the enum: invariant type arguments still differ in both directions.
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

/** @param Box<Suit::A|Suit::B> $b */
function takePartial(Box $b): void {}

function check(): void {
    /** @var Box<Suit::A|Suit::B> $partial */
    $partial = new Box(Suit::A);
    takeEnum($partial);
//           ^^^^^^^^ InvalidArgument: Argument $b of takeEnum() expects 'Box<Suit>', got 'Box<Suit::A|Suit::B>'
    /** @var Box<Suit> $enum */
    $enum = new Box(Suit::A);
    takePartial($enum);
//              ^^^^^ InvalidArgument: Argument $b of takePartial() expects 'Box<Suit::A|Suit::B>', got 'Box<Suit>'
}
