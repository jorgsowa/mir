===description===
A private instance method used only through first-class-callable syntax
(`$this->helper(...)`) must not be reported unused.
===config===
suppress=
===file===
<?php
class Foo {
    private function helper(): void {}

    public function run(): void {
        ($this->helper(...))();
    }
}
===expect===
