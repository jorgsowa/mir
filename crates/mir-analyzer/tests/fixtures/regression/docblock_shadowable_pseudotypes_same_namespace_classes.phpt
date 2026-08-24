===description===
Shadowable docblock pseudotype aliases should resolve to same-namespace classes
when those classes exist, rather than falling back to scalar pseudotypes.
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
{
}

/**
 * @param Boolean $value
 */
function acceptsBoolean($value): void
{
}

/**
 * @param Double $value
 */
function acceptsDouble($value): void
{
}

acceptsInteger(new Integer());
acceptsInteger(5);
acceptsBoolean(new Boolean());
acceptsBoolean(false);
acceptsDouble(new Double());
acceptsDouble(3.14);
===expect===
UnusedParam@19:24-19:30: Parameter $value is never used
UnusedParam@26:24-26:30: Parameter $value is never used
UnusedParam@33:23-33:29: Parameter $value is never used
InvalidArgument@38:15-38:16: Argument $value of acceptsInteger() expects 'Regression\DocblockTypePrecedence\Integer', got '5'
InvalidArgument@40:15-40:20: Argument $value of acceptsBoolean() expects 'Regression\DocblockTypePrecedence\Boolean', got 'false'
InvalidArgument@42:14-42:18: Argument $value of acceptsDouble() expects 'Regression\DocblockTypePrecedence\Double', got '3.14'
