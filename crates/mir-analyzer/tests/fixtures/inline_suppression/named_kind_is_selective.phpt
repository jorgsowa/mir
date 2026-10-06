===description===
a named @mir-ignore suppresses only that kind, leaving others on the line
===file===
<?php
function test(): void {
    noSuchFunc(new NoSuchClass()); // @mir-ignore UndefinedClass
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedFunction: Function noSuchFunc() is not defined
}
