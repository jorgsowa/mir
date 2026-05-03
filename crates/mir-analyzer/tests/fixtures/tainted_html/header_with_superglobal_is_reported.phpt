===description===
header with superglobal is reported
===file===
<?php
function redirect(): void {
    header('Location: ' . $_GET['next']);
}
===expect===
TaintedHtml@3:4: Tainted HTML output — possible XSS
===ignore===
TODO
