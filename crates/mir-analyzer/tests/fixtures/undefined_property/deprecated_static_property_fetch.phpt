===description===
deprecatedStaticPropertyFetch
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
DeprecatedProperty
===ignore===
TODO
