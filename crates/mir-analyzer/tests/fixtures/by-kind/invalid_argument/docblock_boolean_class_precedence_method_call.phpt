===description===
Method-call argument checking should respect class precedence for a shadowable
docblock pseudotype alias on the method parameter.
===file===
<?php
namespace Regression\DocblockTypePrecedence;

final class Boolean
{
}

final class Handler
{
    /**
     * @param Boolean $value
     */
    public function accepts($value): void
    {
    }
}

$handler = new Handler();
$handler->accepts(new Boolean());
$handler->accepts(false);
===expect===
UnusedParam@13:28-13:34: Parameter $value is never used
InvalidArgument@20:18-20:23: Argument $value of accepts() expects 'Regression\DocblockTypePrecedence\Boolean', got 'false'
