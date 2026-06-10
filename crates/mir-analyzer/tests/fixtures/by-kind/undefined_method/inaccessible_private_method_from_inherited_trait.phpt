===description===
Inaccessible private method from inherited trait
===ignore===
TODO
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
