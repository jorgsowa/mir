===description===
@psalm-internal is recognized as an alias for @internal (the namespace
argument is accepted but not itself enforced, matching bare @internal).
===file:Library.php===
<?php
namespace Vendor\Library;

class Foo {
    /**
     * @psalm-internal Vendor\Library
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
Main.php: InternalMethod@4:0-4:22: Method Vendor\Library\Foo::internalHelper() is marked @internal
