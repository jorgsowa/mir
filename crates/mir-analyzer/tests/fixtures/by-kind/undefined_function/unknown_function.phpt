===description===
unknown function
===file===
<?php
function test(): void {
    foo();
}
===expect===
UndefinedFunction@3:5-3:10: Function foo() is not defined
