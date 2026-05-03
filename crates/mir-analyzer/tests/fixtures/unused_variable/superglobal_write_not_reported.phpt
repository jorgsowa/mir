===description===
superglobal write not reported
===file===
<?php
function test(): void {
    $_GET['debug'] = '1';
}
===expect===
===ignore===
TODO
