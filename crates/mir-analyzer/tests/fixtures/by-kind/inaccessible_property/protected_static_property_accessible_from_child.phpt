===description===
InaccessibleProperty does NOT fire when a subclass accesses a protected static property declared on its parent via self::.
===file===
<?php
class Base
{
    protected static int $value = 1;
}

class Child extends Base
{
    public static function getValue(): int
    {
        return self::$value;
    }
}
===expect===
