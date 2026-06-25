===description===
DeprecatedProperty fires when accessing a deprecated static property.
===file===
<?php
class App {
    /** @deprecated use $instance instead */
    public static string $old = "v1";
}

echo App::$old;
===expect===
DeprecatedProperty@7:10-7:14: Property App::$old is deprecated: use $instance instead
