===description===
global namespace unknown function
===file===
<?php
function test(): void {
    \nonExistent();
}
===expect===
UndefinedFunction: Function nonExistent() is not defined
===ignore===
TODO
