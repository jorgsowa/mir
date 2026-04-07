===source===
<?php
function test(): void {
    foo();
    foo();
}
===expect===
UndefinedFunction at 3:4
UndefinedFunction at 4:4
