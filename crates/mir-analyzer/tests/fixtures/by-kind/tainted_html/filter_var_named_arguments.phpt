===description===
Named arguments to the filter functions are matched by name, in any order.
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
function value_named_last(): void {
    echo filter_var(filter: FILTER_DEFAULT, value: $_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function value_named_first(): void {
    echo filter_var(value: $_GET['x'], filter: FILTER_DEFAULT);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function named_validating_filter_clears(): void {
    echo filter_var(filter: FILTER_VALIDATE_INT, value: $_GET['x']);
}
