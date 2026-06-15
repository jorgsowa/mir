===description===
global namespace unknown function
===file===
<?php
function test(): void {
    \nonExistent();
}
===expect===
UndefinedFunction@3:4-3:18: Function nonExistent() is not defined
