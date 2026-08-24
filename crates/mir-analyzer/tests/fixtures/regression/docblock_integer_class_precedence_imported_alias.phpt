===description===
An imported class alias should also take precedence over the shadowable
`integer` docblock pseudotype spelling.
===file:main.php===
<?php
namespace Regression\DocblockTypePrecedence;

use Regression\DocblockTypePrecedence\Support\Integer;

/**
 * @param Integer $value
 */
function acceptsIntegerAlias($value): void
{
}

acceptsIntegerAlias(new Integer());
acceptsIntegerAlias(5);
===file:Support/Integer.php===
<?php
namespace Regression\DocblockTypePrecedence\Support;

final class Integer
{
}
===expect===
main.php: UnusedParam@9:29-9:35: Parameter $value is never used
main.php: InvalidArgument@14:20-14:21: Argument $value of acceptsIntegerAlias() expects 'Regression\DocblockTypePrecedence\Support\Integer', got '5'
