===description===
multiple call sites
===file===
<?php
function test(): void {
    foo();
    foo();
}
===expect===
UndefinedFunction@3:4: Function foo() is not defined
UndefinedFunction@4:4: Function foo() is not defined
