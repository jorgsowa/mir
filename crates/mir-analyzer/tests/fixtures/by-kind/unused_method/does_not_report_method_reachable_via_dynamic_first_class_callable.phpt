===description===
A private method reachable only via a dynamic first-class-callable
($this->$name(...)) must not be reported unused, mirroring the ordinary
dynamic call ($this->$name()) exemption.
===config===
suppress=
===file===
<?php
class Foo {
    private function helper(): void {}

    public function run(string $name): callable {
        return $this->$name(...);
    }
}
===expect===
