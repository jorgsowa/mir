===description===
`decimal-int-string`, `non-decimal-int-string` and `non-empty-literal-string` are strings, not classes.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

/** @param decimal-int-string $value */
function acceptsDecimal($value): void {}

/** @param non-decimal-int-string $value */
function acceptsNonDecimal($value): void {}

/** @param non-empty-literal-string $value */
function acceptsNonEmptyLiteral($value): void {}

acceptsDecimal('123');
acceptsDecimal('-1');
acceptsDecimal('abc');
//             ^^^^^ InvalidArgument: Argument $value of acceptsDecimal() expects 'decimal-int-string', got '"abc"'

acceptsNonDecimal('00');
acceptsNonDecimal('foo');
acceptsNonDecimal(12);
//                ^^ ArgumentTypeCoercion: Argument $value of acceptsNonDecimal() expects 'non-decimal-int-string', got '12' — coercion may fail at runtime

acceptsNonEmptyLiteral('x');
acceptsNonEmptyLiteral('');
//                     ^^ InvalidArgument: Argument $value of acceptsNonEmptyLiteral() expects 'non-empty-literal-string', got '""'
