===description===
An enum's `@implements Interface<ConcreteType>` type args are now recognized
when inferring a template through the implements chain — `EnumDef` had no
`implements_type_args` field at all, so this always fell back to `mixed`.
===config===
suppress=MissingPropertyType,UnusedParam,MissingThrowsDocblock,UnusedVariable
===file===
<?php
/** @template T */
interface HasColor {}

/** @implements HasColor<string> */
enum Suit implements HasColor {
    case Hearts;
    case Spades;

    public function color(): string {
        return match ($this) {
            self::Hearts => 'red',
            self::Spades => 'black',
        };
    }
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
