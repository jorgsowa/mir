===description===
Method parameter types prefer matching classes over docblock pseudotype aliases.
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
//                          ^^^^^^ UnusedParam: Parameter $value is never used
    {
    }
}

$handler = new Handler();
$handler->accepts(new Boolean());
$handler->accepts(false);
//                ^^^^^ InvalidArgument: Argument $value of accepts() expects 'Regression\DocblockTypePrecedence\Boolean', got 'false'
===expect===
