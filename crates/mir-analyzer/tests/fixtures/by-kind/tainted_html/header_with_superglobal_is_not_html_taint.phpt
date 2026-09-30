===description===
`header()` is not an HTML output sink.
===config===
suppress=MixedArrayAccess
===file===
<?php
function redirect(): void {
    header('Location: ' . $_GET['next']);
}
===expect===
