===description===
Deprecated static property fetch
===file===
<?php

class Bar
{
    /**
     * @deprecated
     */
    public static bool $deprecatedProperty = false;
}

Bar::$deprecatedProperty;

===expect===
DeprecatedProperty@11:6-11:25: Property Bar::$deprecatedProperty is deprecated
