===description===
does not count call after switch break
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
UnreachableCode@7:17: Unreachable code detected
UnusedMethod@11:4: Private method Foo::helper() is never called
