===description===
unknown function
===file===
<?php
function test(): void {
    foo();
//  ^^^^^ UndefinedFunction: Function foo() is not defined
}
===expect===
