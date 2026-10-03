===description===
A docblock case union nested in a generic return type equals the enum only when complete; a partial union is rejected.
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

/** @return Box<Suit> */
function complete(): Box {
    /** @var Box<Suit::A|Suit::B|Suit::C> $b */
    $b = new Box(Suit::A);
    return $b;
}

/** @return Box<Suit::A|Suit::B> */
function partial(): Box {
    /** @var Box<Suit> $b */
    $b = new Box(Suit::A);
    return $b;
//  ^^^^^^^^^^ InvalidReturnType: Return type 'Box<Suit>' is not compatible with declared 'Box<Suit::A|Suit::B>'
}
===expect===
