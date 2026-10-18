===description===
A printf-style format only passes on the taint of values it interpolates as
text (`%s`); numeric conversions can't carry a payload. A format that isn't
a literal passes everything on. `json_encode()` with `JSON_HEX_TAG` can't
open an HTML tag.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function formats(string $format): void {
    echo sprintf('%d items', $_GET['n']);
    echo sprintf('%05.2f%%', $_GET['price']);
    echo sprintf('%s items', $_GET['n']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo sprintf('%2$d of %1$s', $_GET['label'], 3);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo sprintf('%1$s of %2$d', 'Page', $_GET['page']);
    echo sprintf($format, $_GET['n']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo vsprintf('%d-%d', $_GET['range']);
    printf('%d results', $_GET['n']);
    printf('Hello %s', $_GET['name']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    vprintf('%s', $_GET['parts']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function json(): void {
    echo json_encode($_GET);
//  ^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo json_encode($_GET, JSON_HEX_TAG | JSON_HEX_AMP);
    echo json_encode($_GET, JSON_PRETTY_PRINT);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
