===description===
Docblock pseudotypes retain their scalar meaning without a matching class.
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
