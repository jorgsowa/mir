===description===
InaccessibleClassConstant fires when accessing a protected constant via a child class reference from outside the hierarchy.
===file===
<?php
class Base {
    protected const CONFIG = "internal";
}

class Child extends Base {}

echo Child::CONFIG;
===expect===
InaccessibleClassConstant@8:12-8:18: Cannot access constant Child::CONFIG
