===source===
<?php
function test(): void {
    throw new \RuntimeException('something went wrong');
}
===expect===
