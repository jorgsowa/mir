===description===
Does not report internal method called from same namespace
===file===
<?php
namespace Vendor\Library;

class Foo {
    /**
     * @internal
     */
    public function internalHelper(): void {
    }
}

$foo = new Foo();
$foo->internalHelper();
===expect===
