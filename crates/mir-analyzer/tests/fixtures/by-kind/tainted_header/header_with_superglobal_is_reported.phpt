===description===
Tainted input in `header()` reports TaintedHeader, not TaintedHtml.
===config===
suppress=MixedArrayAccess
===file===
<?php
function redirect(): void {
    header('Location: ' . $_GET['next']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHeader: Tainted HTTP header — possible header injection or open redirect
}
===expect===
