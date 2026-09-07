===description===
When a parent method's docblock uses a shadowable pseudotype alias that is
also a real class in scope, inherited method calls must keep enforcing the
class-resolved parameter type.
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
    {
    }
}

final class ChildHandler extends ParentHandler
{
}

$handler = new ChildHandler();
$handler->accepts(new Integer());
$handler->accepts(5);
===expect===
UnusedParam@13:28-13:34: Parameter $value is never used
InvalidArgument@24:18-24:19: Argument $value of accepts() expects 'Regression\DocblockTypePrecedence\Integer', got '5'
