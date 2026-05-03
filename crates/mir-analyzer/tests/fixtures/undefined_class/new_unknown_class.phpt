===description===
new unknown class
===file===
<?php
function test(): void {
    new UnknownClass();
}
===expect===
UndefinedClass@3:8: Class UnknownClass does not exist
===ignore===
TODO
