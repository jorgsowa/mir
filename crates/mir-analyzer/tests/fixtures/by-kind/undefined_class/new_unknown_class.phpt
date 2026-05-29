===description===
new unknown class
===file===
<?php
function test(): void {
    new UnknownClass();
}
===expect===
UndefinedClass@3:9-3:21: Class UnknownClass does not exist
