===description===
A known sanitizer method clears its argument when the receiver is built in place.
===config===
<mir>
  <issueHandlers>
    <MissingParamType errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class HTMLPurifier { public function purify($h) { return $h; } }
echo (new HTMLPurifier())->purify($_GET['x']);
echo (new HTMLPurifier())->purify($_GET['x']) . $_GET['y'];
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
