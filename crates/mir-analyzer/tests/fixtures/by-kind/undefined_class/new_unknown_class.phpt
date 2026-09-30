===description===
new unknown class
===file===
<?php
function test(): void {
    new UnknownClass();
//      ^^^^^^^^^^^^ UndefinedClass: Class UnknownClass does not exist
}
===expect===
