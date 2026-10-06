===description===
inline suppression reaches dead-code diagnostics (cross-pass)
===file===
<?php
class C {
    // @mir-ignore UnusedMethod
    private function a(): void {}
    private function b(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedMethod: Private method C::b() is never called
}
