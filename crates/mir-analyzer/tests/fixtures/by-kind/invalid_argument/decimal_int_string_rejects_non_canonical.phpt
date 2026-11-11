===description===
`decimal-int-string` accepts only the canonical spelling of an int, which PHP casts to an int array key: `'007'` and `'+1'` are rejected. `non-decimal-int-string` rejects exactly those canonical spellings. A `decimal-int-string` is a `numeric-string`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

/** @param decimal-int-string $value */
function decimal(string $value): void {}

/** @param non-decimal-int-string $value */
function nonDecimal(string $value): void {}

/** @param numeric-string $value */
function numeric(string $value): void {}

decimal('123');
decimal('-1');
decimal('0');
decimal('007');
//      ^^^^^ InvalidArgument: Argument $value of decimal() expects 'decimal-int-string', got '"007"'
decimal('+1');
//      ^^^^ InvalidArgument: Argument $value of decimal() expects 'decimal-int-string', got '"+1"'

nonDecimal('00');
nonDecimal('1.2');
nonDecimal('foo');
nonDecimal('123');
//         ^^^^^ InvalidArgument: Argument $value of nonDecimal() expects 'non-decimal-int-string', got '"123"'
nonDecimal('-1');
//         ^^^^ InvalidArgument: Argument $value of nonDecimal() expects 'non-decimal-int-string', got '"-1"'

/** @param decimal-int-string $value */
function forwards(string $value): void {
    numeric($value);
}
