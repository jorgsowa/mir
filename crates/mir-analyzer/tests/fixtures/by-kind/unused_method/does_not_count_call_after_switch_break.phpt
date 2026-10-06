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
//              ^^^^^^^^^^^^^^^^ UnreachableCode: Unreachable code detected
        }
    }

    private function helper(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedMethod: Private method Foo::helper() is never called
}
