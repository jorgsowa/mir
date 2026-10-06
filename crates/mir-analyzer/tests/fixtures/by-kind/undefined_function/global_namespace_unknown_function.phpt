===description===
global namespace unknown function
===file===
<?php
function test(): void {
    \nonExistent();
//  ^^^^^^^^^^^^^^ UndefinedFunction: Function nonExistent() is not defined
}
