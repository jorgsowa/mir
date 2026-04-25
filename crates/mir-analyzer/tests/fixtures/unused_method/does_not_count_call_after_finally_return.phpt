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
UnreachableCode: Unreachable code detected
UnusedMethod: Private method Foo::helper() is never called
