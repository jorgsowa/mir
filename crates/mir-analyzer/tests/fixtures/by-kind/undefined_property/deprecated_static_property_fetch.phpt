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
//   ^^^^^^^^^^^^^^^^^^^ DeprecatedProperty: Property Bar::$deprecatedProperty is deprecated

===expect===
