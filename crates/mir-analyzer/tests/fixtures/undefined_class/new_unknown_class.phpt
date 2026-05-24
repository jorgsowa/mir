===description===
new unknown class
===file===
<?php
function test(): void {
    new UnknownClass();
}
===expect===
UndefinedClass@3:9: Class UnknownClass does not exist
