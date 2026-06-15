===description===
unknown function
===file===
<?php
function test(): void {
    foo();
}
===expect===
UndefinedFunction@3:4-3:9: Function foo() is not defined
