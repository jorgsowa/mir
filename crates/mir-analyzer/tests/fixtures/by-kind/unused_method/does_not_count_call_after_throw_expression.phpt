===description===
does not count call after throw expression
===file===
<?php
class Foo {
    public function run(): void {
        $value = throw new RuntimeException('stop');
        $this->helper();
    }

    private function helper(): void {}
}
===expect===
UnreachableCode@5:8-5:24: Unreachable code detected
UnusedMethod@8:4-8:38: Private method Foo::helper() is never called
