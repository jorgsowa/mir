===description===
extract() defines variables whose names are only known at runtime, from
the keys of its source array — but a tainted source array (`extract($_GET)`)
never made any of those variables taint-tracked, silently producing no
diagnostic when one was later echoed.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function viaExtract(): void {
    extract($_GET);
    echo $name;
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function safeExtractOnly(): void {
    $vars = ['name' => 'literal'];
    extract($vars);
    echo $name;
}
===expect===
