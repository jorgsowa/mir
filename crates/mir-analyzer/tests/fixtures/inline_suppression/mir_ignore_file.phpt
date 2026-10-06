===description===
@mir-ignore-file suppresses the named kind across the whole file
===file===
<?php
// @mir-ignore-file UndefinedClass
function test(): void {
    new NoSuchClass();
    new AlsoMissing();
    noSuchFunc();
//  ^^^^^^^^^^^^ UndefinedFunction: Function noSuchFunc() is not defined
}
