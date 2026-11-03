===description===
a multi-line @psalm-suppress docblock above a declaration reaches the declaration (skips the closing */)
===file===
<?php
class C {
//<^ +6:1 UnusedClass: Class C is never referenced
    /**
     * @psalm-suppress UnusedMethod
     */
    private function a(): void {}
    private function b(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedMethod: Private method C::b() is never called
}
