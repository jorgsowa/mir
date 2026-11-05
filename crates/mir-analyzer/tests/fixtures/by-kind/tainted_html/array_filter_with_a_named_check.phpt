===description===
`array_filter` with a named value check keeps only elements that check accepts.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function numeric(): void {
    echo implode(',', array_filter($_GET, 'is_numeric'));
}

function digits(): void {
    echo implode(',', array_filter($_GET, 'ctype_digit'));
}

function otherCallback(): void {
    echo implode(',', array_filter($_GET, 'strlen'));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function keyMode(): void {
    echo implode(',', array_filter($_GET, 'ctype_digit', ARRAY_FILTER_USE_KEY));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
