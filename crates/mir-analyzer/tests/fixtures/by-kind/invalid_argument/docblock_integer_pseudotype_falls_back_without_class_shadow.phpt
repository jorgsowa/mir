===description===
Docblock pseudotypes retain their scalar meaning without a matching class.
===file===
<?php
namespace Regression\DocblockTypePrecedence;

/**
 * @param integer $value
 */
function acceptsIntegerPseudo($value): void
//                            ^^^^^^ UnusedParam: Parameter $value is never used
{
}

acceptsIntegerPseudo(5);
