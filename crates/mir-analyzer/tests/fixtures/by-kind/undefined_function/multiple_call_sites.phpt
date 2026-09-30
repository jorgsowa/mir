===description===
multiple call sites
===file===
<?php
function test(): void {
    foo();
//  ^^^^^ UndefinedFunction: Function foo() is not defined
    foo();
//  ^^^^^ UndefinedFunction: Function foo() is not defined
}
===expect===
