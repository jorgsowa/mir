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
    }

    private function helper(): void {}
}
===expect===
UnreachableCode@10:9-10:25: Unreachable code detected
UnusedMethod@13:4-13:38: Private method Foo::helper() is never called
