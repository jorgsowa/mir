===description===
does not count call after finally return
===file===
<?php
class Foo {
    public function run(): void {
        try {
            echo 'work';
        } finally {
            return;
        }

        $this->helper();
//      ^^^^^^^^^^^^^^^^ UnreachableCode: Unreachable code detected
    }

    private function helper(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedMethod: Private method Foo::helper() is never called
}
