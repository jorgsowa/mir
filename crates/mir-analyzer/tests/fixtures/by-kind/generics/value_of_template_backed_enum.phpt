===description===
`value-of<T>` with `T` bound to a backed enum is the backing value of the case passed.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

enum Suit: string
{
    case Hearts = 'hearts';
    case Spades = 'spades';
}

/**
 * @template T of Suit
 * @param T $case
 * @return value-of<T>
 */
function backingValue(Suit $case)
{
    return $case->value;
}

/** @param 'hearts' $value */
function acceptsHearts(string $value): void {}

function acceptsString(string $value): void {}

function acceptsInt(int $value): void {}

acceptsString(backingValue(Suit::Hearts));
acceptsHearts(backingValue(Suit::Hearts));
acceptsHearts(backingValue(Suit::Spades));
//            ^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $value of acceptsHearts() expects '"hearts"', got '"spades"'
acceptsInt(backingValue(Suit::Spades));
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $value of acceptsInt() expects 'int', got '"spades"'
