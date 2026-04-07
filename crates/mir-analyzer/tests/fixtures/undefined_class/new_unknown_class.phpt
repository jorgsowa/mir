===source===
<?php
function test(): void {
    new UnknownClass();
}
===expect===
UndefinedClass: UnknownClass
