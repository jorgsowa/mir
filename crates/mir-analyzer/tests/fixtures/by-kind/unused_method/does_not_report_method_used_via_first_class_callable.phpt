===description===
A private instance method used only through first-class-callable syntax
(`$this->helper(...)`) must not be reported unused.
===config===
suppress=UnusedVariable
===file===
<?php
class Foo {
    private function helper(): void {}

    public function run(): void {
        $c = $this->helper(...);
        $c();
    }
}
===expect===
