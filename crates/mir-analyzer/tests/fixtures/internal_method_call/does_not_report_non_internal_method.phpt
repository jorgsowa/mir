===description===
Does not report non-internal method
===file===
<?php
namespace Vendor\Library;

class Foo {
    public function publicHelper(): void {
    }
}

// In user code from different namespace
namespace User;
$foo = new \Vendor\Library\Foo();
$foo->publicHelper();
===expect===
