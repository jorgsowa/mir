===description===
multiple call sites
===file===
<?php
function test(): void {
    foo();
    foo();
}
===expect===
UndefinedFunction@3:5: Function foo() is not defined
UndefinedFunction@4:5: Function foo() is not defined
