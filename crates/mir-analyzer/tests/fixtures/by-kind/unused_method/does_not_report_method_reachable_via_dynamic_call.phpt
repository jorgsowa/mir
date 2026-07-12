===description===
A private method reachable only via a dynamic method call ($this->$name())
elsewhere on the class must not be reported unused, since the exact target
isn't statically known.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
class Foo {
    private function helper(): void {}

    public function run(string $name): void {
        $this->$name();
    }
}
===expect===
