===file===
<?php
function test(): void {
    array_map(fn($x) => $x, [1, 2, 3]);
}
===expect===
