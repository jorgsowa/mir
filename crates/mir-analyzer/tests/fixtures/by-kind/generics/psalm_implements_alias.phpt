===description===
@psalm-implements is recognized as an alias for @implements (the vendor
prefix was previously only accepted for @phpstan-implements, not
@psalm-implements).
===config===
suppress=MissingPropertyType,UnusedParam,MissingThrowsDocblock,UnusedVariable
===file===
<?php
/** @template T */
interface HasColor {}

/** @psalm-implements HasColor<string> */
enum Suit implements HasColor {
    case Hearts;
    case Spades;
}

/**
 * @template T
 * @param HasColor<T> $c
 * @return T
 */
function colorOf(HasColor $c) {
    throw new \Exception();
}

$x = colorOf(Suit::Hearts);
/** @mir-check $x is string */
echo "ok";
===expect===
