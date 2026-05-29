===description===
global namespace unknown function
===file===
<?php
function test(): void {
    \nonExistent();
}
===expect===
UndefinedFunction@3:5-3:19: Function nonExistent() is not defined
