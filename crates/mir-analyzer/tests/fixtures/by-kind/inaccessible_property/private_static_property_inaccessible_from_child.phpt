===description===
InaccessibleProperty fires when a subclass accesses a private static property declared on its parent via self::.
===file===
<?php
class Base
{
    private static string $secret = 'hidden';
}

class Child extends Base
{
    public static function getSecret(): string
    {
        return self::$secret;
    }
}
===expect===
InaccessibleProperty@11:21-11:28: Cannot access property Base::$secret
