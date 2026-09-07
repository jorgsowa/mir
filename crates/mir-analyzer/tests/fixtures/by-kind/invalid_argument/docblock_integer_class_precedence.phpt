===description===
When a same-named class is in scope, a docblock `@param Integer` must resolve
to that class rather than to the `integer` pseudo-type alias.
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
{
}

acceptsIntegerClass(new Integer());
acceptsIntegerClass(5);
===expect===
UnusedParam@11:29-11:35: Parameter $value is never used
InvalidArgument@16:20-16:21: Argument $value of acceptsIntegerClass() expects 'Regression\DocblockTypePrecedence\Integer', got '5'
