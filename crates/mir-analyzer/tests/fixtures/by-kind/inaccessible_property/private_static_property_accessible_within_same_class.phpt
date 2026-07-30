===description===
InaccessibleProperty does NOT fire when a private static property is accessed from within its declaring class via self::.
===file===
<?php
class Config
{
    private static string $secret = 'hidden';

    public static function getSecret(): string
    {
        return self::$secret;
    }
}
===expect===
