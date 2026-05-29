===description===
multiple call sites
===file===
<?php
function test(): void {
    foo();
    foo();
}
===expect===
UndefinedFunction@3:5-3:10: Function foo() is not defined
UndefinedFunction@4:5-4:10: Function foo() is not defined
