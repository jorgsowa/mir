===description===
InaccessibleProperty fires when accessing a protected property from outside the class hierarchy.
===file===
<?php
class Config
{
    protected string $internal = 'hidden';
}

echo (new Config())->internal;
===expect===
InaccessibleProperty@7:21-7:29: Cannot access property Config::$internal
