===source===
<?php
function test(): void {
    \nonExistent();
}
===expect===
UndefinedFunction at 3:4
