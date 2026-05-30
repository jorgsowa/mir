===description===
does not report exception
===file===
<?php
function test(): void {
    throw new \RuntimeException('something went wrong');
}
===expect===
