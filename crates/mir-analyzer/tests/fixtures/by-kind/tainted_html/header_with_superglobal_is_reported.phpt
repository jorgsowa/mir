===description===
header with superglobal is reported
===config===
suppress=MixedArrayAccess
===file===
<?php
function redirect(): void {
    header('Location: ' . $_GET['next']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
===expect===
