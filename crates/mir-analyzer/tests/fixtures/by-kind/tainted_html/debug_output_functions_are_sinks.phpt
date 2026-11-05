===description===
These print their argument, unless the return flag makes them hand it back.
===config===
<mir>
  <issueHandlers>
    <MissingParamType errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedMethodCall errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <UndefinedClass errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function printed(): void {
    print_r($_GET);
//  ^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    var_dump($_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    var_export($_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    highlight_string($_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    print_r($_GET, false);
//  ^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function returned(bool $flag): void {
    $a = print_r($_GET, true);
    $b = var_export($_GET['x'], true);
    $c = highlight_string($_GET['x'], true);
    $d = print_r($_GET, $flag);
    var_dump('a');
}
