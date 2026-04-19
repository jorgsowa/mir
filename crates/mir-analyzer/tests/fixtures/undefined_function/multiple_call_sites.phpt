===source===
<?php
function test(): void {
    foo();
    foo();
}
===expect===
UndefinedFunction: Function foo() is not defined
UndefinedFunction: Function foo() is not defined
