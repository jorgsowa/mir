===description===
InaccessibleProperty fires when accessing a private static property through its class name from outside.
===file===
<?php
class Config
{
    private static string $secret = 'hidden';
}

echo Config::$secret;
//           ^^^^^^^ InaccessibleProperty: Cannot access property Config::$secret
===expect===
