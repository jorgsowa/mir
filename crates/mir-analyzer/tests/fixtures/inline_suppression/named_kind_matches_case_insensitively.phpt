===description===
a named @mir-ignore matches its kind case-insensitively — undefinedclass
still suppresses UndefinedClass
===file===
<?php
function test(): void {
    noSuchFunc(new NoSuchClass()); // @mir-ignore undefinedclass
}
===expect===
UndefinedFunction@3:4-3:33: Function noSuchFunc() is not defined
