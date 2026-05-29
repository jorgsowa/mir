===description===
Reports internal static method called from different namespace
===file:Library.php===
<?php
namespace Vendor\Library;

class Foo {
    /**
     * @internal
     */
    public static function internalHelper(): void {
    }
}
===file:Main.php===
<?php
namespace User;
\Vendor\Library\Foo::internalHelper();
===expect===
Main.php: InternalMethod@3:1-3:38: Method Vendor\Library\Foo::internalHelper() is marked @internal
