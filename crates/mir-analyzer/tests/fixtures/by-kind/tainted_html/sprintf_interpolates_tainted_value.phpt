===description===
sprintf()/vsprintf() interpolate every argument straight into the
returned string, but neither was ever modeled as a taint pass-through —
echoing their result with a tainted argument silently produced no
diagnostic at all.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function viaSprintf(): void {
    echo sprintf('<b>%s</b>', $_GET['name']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function viaVsprintf(): void {
    echo vsprintf('<b>%s</b>', [$_GET['name']]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function safeFormatOnly(): void {
    echo sprintf('<b>%s</b>', 'static');
}
===expect===
