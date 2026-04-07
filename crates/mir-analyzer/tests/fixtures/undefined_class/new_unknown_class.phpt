===source===
<?php
function test(): void {
    new UnknownClass();
}
===expect===
UndefinedClass at 3:8
