===description===
Inside a namespace, an unqualified or fully qualified call to a builtin is
still the builtin: it passes taint through or sanitizes as usual.
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
namespace App;

function render(): void {
    echo trim($_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo \strtoupper($_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo htmlspecialchars($_GET['x']);
    echo \htmlspecialchars($_GET['x']);
    echo \strlen($_GET['x']);
}
