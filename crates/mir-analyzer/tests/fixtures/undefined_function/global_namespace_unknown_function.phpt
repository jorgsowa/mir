===file===
<?php
function test(): void {
    \nonExistent();
}
===expect===
UndefinedFunction: Function nonExistent() is not defined
