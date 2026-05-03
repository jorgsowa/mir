===description===
does not count call after switch break
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
UnusedMethod@1:0: Private method Foo::helper() is never called
UnreachableCode@7:16: Unreachable code detected
===ignore===
TODO
