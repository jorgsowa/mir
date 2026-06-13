===description===
header with superglobal is reported
===config===
suppress=MixedArrayAccess
===file===
<?php
function redirect(): void {
    header('Location: ' . $_GET['next']);
}
===expect===
TaintedHtml@3:5-3:41: Tainted HTML output — possible XSS
