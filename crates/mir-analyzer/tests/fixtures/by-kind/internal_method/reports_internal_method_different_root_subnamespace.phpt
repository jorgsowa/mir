===description===
Reports internal method called from a sub-namespace under a different root namespace
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
===file:App.php===
<?php
namespace App\Service;
$foo = new \Vendor\Library\Foo();
$foo->internalHelper();
===expect===
App.php: InternalMethod@4:0-4:22: Method Vendor\Library\Foo::internalHelper() is marked @internal
