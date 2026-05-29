===description===
Reports internal method called from different namespace
===file:Library.php===
<?php
namespace Vendor\Library;

class Foo {
    /**
     * @internal
     */
    public function internalHelper(): void {
    }
}
===file:Main.php===
<?php
namespace User;
$foo = new \Vendor\Library\Foo();
$foo->internalHelper();
===expect===
Main.php: InternalMethod@4:1-4:23: Method Vendor\Library\Foo::internalHelper() is marked @internal
