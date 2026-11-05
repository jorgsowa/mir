===description===
A project's own `request()` or `e()` is read for what it does instead of being treated as Laravel's helper.
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
function request(string $k): string {
//               ^^^^^^^^^ UnusedParam: Parameter $k is never used
    return 'static';
}

function e(string $v): string {
    return $v;
}

echo request('q');
echo e($_GET['x']);
//<^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
