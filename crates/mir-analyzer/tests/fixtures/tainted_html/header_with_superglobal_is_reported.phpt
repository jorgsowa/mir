===file===
<?php
function redirect(): void {
    header('Location: ' . $_GET['next']);
}
===expect===
TaintedHtml: Tainted HTML output — possible XSS
