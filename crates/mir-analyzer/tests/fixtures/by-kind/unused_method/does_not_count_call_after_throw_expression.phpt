===description===
does not count call after throw expression
===file===
<?php
class Foo {
    public function run(): void {
        $value = throw new RuntimeException('stop');
        $this->helper();
//      ^^^^^^^^^^^^^^^^ UnreachableCode: Unreachable code detected
    }

    private function helper(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedMethod: Private method Foo::helper() is never called
}
===expect===
