===description===
InaccessibleProperty fires when accessing a private static property through its class name from outside.
===file===
<?php
class Config
{
    private static string $secret = 'hidden';
}

echo Config::$secret;
===expect===
InaccessibleProperty@7:13-7:20: Cannot access property Config::$secret
