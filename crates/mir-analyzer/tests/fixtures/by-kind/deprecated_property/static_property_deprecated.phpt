===description===
DeprecatedProperty fires when accessing a deprecated static property.
===file===
<?php
class App {
    /** @deprecated use $instance instead */
    public static string $old = "v1";
}

echo App::$old;
//        ^^^^ DeprecatedProperty: Property App::$old is deprecated: use $instance instead
===expect===
