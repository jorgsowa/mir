===description===
does not count call after finally return
===config===
find_dead_code=true
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
UnusedMethod@1:0: Private method Foo::helper() is never called
UnreachableCode@10:8: Unreachable code detected
