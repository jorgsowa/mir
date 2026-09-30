===description===
Docblock types prefer matching in-scope classes over pseudotype aliases.
===file===
<?php
namespace Regression\DocblockTypePrecedence;

final class Integer
{
}

/**
 * @param Integer $value
 */
function acceptsIntegerClass($value): void
//                           ^^^^^^ UnusedParam: Parameter $value is never used
{
}

acceptsIntegerClass(new Integer());
acceptsIntegerClass(5);
//                  ^ InvalidArgument: Argument $value of acceptsIntegerClass() expects 'Regression\DocblockTypePrecedence\Integer', got '5'
===expect===
