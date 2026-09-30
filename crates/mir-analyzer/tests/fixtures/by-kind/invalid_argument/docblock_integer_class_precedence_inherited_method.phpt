===description===
Inherited methods retain class-resolved docblock parameter types.
===file===
<?php
namespace Regression\DocblockTypePrecedence;

final class Integer
{
}

class ParentHandler
{
    /**
     * @param Integer $value
     */
    public function accepts($value): void
//                          ^^^^^^ UnusedParam: Parameter $value is never used
    {
    }
}

final class ChildHandler extends ParentHandler
{
}

$handler = new ChildHandler();
$handler->accepts(new Integer());
$handler->accepts(5);
//                ^ InvalidArgument: Argument $value of accepts() expects 'Regression\DocblockTypePrecedence\Integer', got '5'
===expect===
