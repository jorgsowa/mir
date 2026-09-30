===description===
Imported class aliases take precedence over docblock pseudotype aliases.
===file:main.php===
<?php
namespace Regression\DocblockTypePrecedence;

use Regression\DocblockTypePrecedence\Support\Integer;

/**
 * @param Integer $value
 */
function acceptsIntegerAlias($value): void
//                           ^^^^^^ UnusedParam: Parameter $value is never used
{
}

acceptsIntegerAlias(new Integer());
acceptsIntegerAlias(5);
//                  ^ InvalidArgument: Argument $value of acceptsIntegerAlias() expects 'Regression\DocblockTypePrecedence\Support\Integer', got '5'
===file:Support/Integer.php===
<?php
namespace Regression\DocblockTypePrecedence\Support;

final class Integer
{
}
===expect===
