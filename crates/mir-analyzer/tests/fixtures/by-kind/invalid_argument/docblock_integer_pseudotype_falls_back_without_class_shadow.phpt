===description===
Without an in-scope class named `Integer`, the `integer` docblock spelling
should keep its scalar pseudotype meaning and accept an int argument.
===file===
<?php
namespace Regression\DocblockTypePrecedence;

/**
 * @param integer $value
 */
function acceptsIntegerPseudo($value): void
{
}

acceptsIntegerPseudo(5);
===expect===
UnusedParam@7:30-7:36: Parameter $value is never used
