===source===
<?php
function test(): void {
    foo();
    foo();
}
===expect===
UndefinedFunction: foo()
UndefinedFunction: foo()
