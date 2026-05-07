===description===
does not count call after throw expression
===config===
find_dead_code=true
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
UnusedMethod@1:0: Private method Foo::helper() is never called
UnreachableCode@5:8: Unreachable code detected
