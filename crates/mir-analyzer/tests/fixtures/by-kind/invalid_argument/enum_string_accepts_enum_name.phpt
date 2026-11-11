===description===
`enum-string` accepts the name of an enum and rejects the name of an ordinary class.
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

enum Suit
{
    case Hearts;
}

final class Card {}

/** @return enum-string */
function returnsEnumString()
{
    return Suit::class;
}

/** @param enum-string $value */
function acceptsEnumString($value): void {}

function acceptsString(string $value): void {}

acceptsString(returnsEnumString());
acceptsEnumString(Suit::class);
acceptsEnumString(returnsEnumString());
acceptsEnumString(Card::class);
//                ^^^^^^^^^^^ InvalidArgument: Argument $value of acceptsEnumString() expects 'enum-string', got 'class-string<App\Card>'
