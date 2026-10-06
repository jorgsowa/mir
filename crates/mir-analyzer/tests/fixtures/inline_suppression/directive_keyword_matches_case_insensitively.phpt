===description===
the directive keyword itself is matched case-insensitively, not just the
kind name that follows it — @MIR-IGNORE suppresses the same as @mir-ignore
===file===
<?php
function test(): void {
    noSuchFunc(new NoSuchClass()); // @MIR-IGNORE undefinedclass
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedFunction: Function noSuchFunc() is not defined
}
