===description===
does not count call after finally return
===file===
<?php
class Foo {
//<^ +12:1 UnusedClass: Class Foo is never referenced
    public function run(): void {
        try {
            echo 'work';
        } finally {
            return;
//          ^^^^^^^ DeprecatedSyntax: Returning from a finally block is deprecated
        }

        $this->helper();
//      ^^^^^^^^^^^^^^^^ UnreachableCode: Unreachable code detected
    }

    private function helper(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedMethod: Private method Foo::helper() is never called
}
