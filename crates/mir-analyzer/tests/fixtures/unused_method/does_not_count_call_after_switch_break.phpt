===config===
find_dead_code=true
===file===
<?php
class Foo {
    public function run(int $mode): void {
        switch ($mode) {
            case 1:
                break;
                $this->helper();
        }
    }

    private function helper(): void {}
}
===expect===
UnreachableCode: Unreachable code detected
UnusedMethod: Private method Foo::helper() is never called
