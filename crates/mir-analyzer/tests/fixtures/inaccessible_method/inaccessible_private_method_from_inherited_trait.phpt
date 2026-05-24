===description===
inaccessiblePrivateMethodFromInheritedTrait
===file===
<?php
trait T {
    private function fooFoo(): void {
    }
}

class B {
    use T;
}

class C extends B {
    public function doFoo(): void {
        $this->fooFoo();
    }
}
===expect===
InaccessibleMethod
===ignore===
TODO
