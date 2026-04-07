===source===
<?php
function test(): void {
    \nonExistent();
}
===expect===
UndefinedFunction: \nonExistent()
