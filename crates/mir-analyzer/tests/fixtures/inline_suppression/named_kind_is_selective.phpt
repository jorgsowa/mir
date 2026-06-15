===description===
a named @mir-ignore suppresses only that kind, leaving others on the line
===file===
<?php
function test(): void {
    noSuchFunc(new NoSuchClass()); // @mir-ignore UndefinedClass
}
===expect===
UndefinedFunction@3:4-3:33: Function noSuchFunc() is not defined
