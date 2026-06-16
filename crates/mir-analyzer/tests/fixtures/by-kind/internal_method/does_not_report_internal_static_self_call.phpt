===description===
Calling an @internal static method via self:: within the same class does not emit InternalMethod
===file===
<?php
namespace Vendor\Library;

class Foo {
    /** @internal */
    protected static function internalStatic(): void {}

    public static function doWork(): void {
        self::internalStatic();
    }
}
===expect===
