===description===
A constant header value is clean.
===file===
<?php
function test(): void {
    header('Content-Type: text/plain');
}
===expect===
