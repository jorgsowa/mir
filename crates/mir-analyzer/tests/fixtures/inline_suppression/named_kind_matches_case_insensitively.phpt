===description===
a named @mir-ignore matches its kind case-insensitively — undefinedclass
still suppresses UndefinedClass
===file===
<?php
function test(): void {
    noSuchFunc(new NoSuchClass()); // @mir-ignore undefinedclass
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedFunction: Function noSuchFunc() is not defined
}
