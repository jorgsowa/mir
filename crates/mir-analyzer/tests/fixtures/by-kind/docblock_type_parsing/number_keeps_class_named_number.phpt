===description===
`Number` written with a capital stays a class name; only lowercase `number` is `int|float`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

final class Number
{
    public function __construct(public string $value) {}
}

/** @return Number */
function makeNumber(): Number
{
    return new Number('1');
}

/** @return number */
function makeFloat()
{
    return 1.5;
}

function takesString(string $value): void {}

takesString(makeNumber()->value);
takesString(makeFloat());
//          ^^^^^^^^^^^ ArgumentTypeCoercion: Argument $value of takesString() expects 'string', got 'int|float' — coercion may fail at runtime
