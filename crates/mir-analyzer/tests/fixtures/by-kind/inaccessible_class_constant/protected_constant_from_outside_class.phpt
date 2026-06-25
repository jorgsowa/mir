===description===
InaccessibleClassConstant fires when accessing a protected constant from outside the class hierarchy.
===file===
<?php
class Config {
    protected const INTERNAL = "hidden";
}

echo Config::INTERNAL;
===expect===
InaccessibleClassConstant@6:13-6:21: Cannot access constant Config::INTERNAL
