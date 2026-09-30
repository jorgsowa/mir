===description===
InaccessibleClassConstant fires when accessing a protected constant from outside the class hierarchy.
===file===
<?php
class Config {
    protected const INTERNAL = "hidden";
}

echo Config::INTERNAL;
//           ^^^^^^^^ InaccessibleClassConstant: Cannot access constant Config::INTERNAL
===expect===
