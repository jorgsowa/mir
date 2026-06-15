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
DeprecatedProperty@11:5-11:24: Property Bar::$deprecatedProperty is deprecated
