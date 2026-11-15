===description===
`assert()` narrows types but keeps taint: production builds set
`zend.assertions=-1`, so the check never runs there.
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
function show(): void {
    $id = $_GET['id'];
    assert(ctype_digit($id));
    echo $id;
//  ^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function showMatched(): void {
    $slug = $_GET['slug'];
    assert(preg_match('/^[a-z]+$/', $slug) === 1);
    echo $slug;
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function showChecked(): void {
    $id = $_GET['id'];
    if (!ctype_digit($id)) {
        return;
    }
    echo $id;
}
