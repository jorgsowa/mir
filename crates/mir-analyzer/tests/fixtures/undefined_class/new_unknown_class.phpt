===source===
<?php
function test(): void {
    new UnknownClass();
}
===expect===
UndefinedClass: Class UnknownClass does not exist
