===description===
Calling an @internal method via $this within the same class does not emit InternalMethod
===file===
<?php
namespace Vendor\Library;

class Foo {
    /** @internal */
    protected function internalHelper(): void {}

    public function doWork(): void {
        $this->internalHelper();
    }
}
===expect===
