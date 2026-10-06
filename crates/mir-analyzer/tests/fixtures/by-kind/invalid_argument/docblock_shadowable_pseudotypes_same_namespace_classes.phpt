===description===
Docblock pseudotype aliases prefer matching classes in the same namespace.
===file===
<?php
namespace Regression\DocblockTypePrecedence;

final class Integer
{
}

final class Boolean
{
}

final class Double
{
}

/**
 * @param Integer $value
 */
function acceptsInteger($value): void
//                      ^^^^^^ UnusedParam: Parameter $value is never used
{
}

/**
 * @param Boolean $value
 */
function acceptsBoolean($value): void
//                      ^^^^^^ UnusedParam: Parameter $value is never used
{
}

/**
 * @param Double $value
 */
function acceptsDouble($value): void
//                     ^^^^^^ UnusedParam: Parameter $value is never used
{
}

acceptsInteger(new Integer());
acceptsInteger(5);
//             ^ InvalidArgument: Argument $value of acceptsInteger() expects 'Regression\DocblockTypePrecedence\Integer', got '5'
acceptsBoolean(new Boolean());
acceptsBoolean(false);
//             ^^^^^ InvalidArgument: Argument $value of acceptsBoolean() expects 'Regression\DocblockTypePrecedence\Boolean', got 'false'
acceptsDouble(new Double());
acceptsDouble(3.14);
//            ^^^^ InvalidArgument: Argument $value of acceptsDouble() expects 'Regression\DocblockTypePrecedence\Double', got '3.14'
