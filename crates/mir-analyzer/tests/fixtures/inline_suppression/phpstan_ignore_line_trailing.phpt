===description===
trailing // @phpstan-ignore-line suppresses its own line
===file===
<?php
function test(): void {
    new NoSuchClass(); // @phpstan-ignore-line
}
===expect===
