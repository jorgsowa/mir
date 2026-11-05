===description===
`CURLOPT_HTTPHEADER` sends the given header lines verbatim, in either form of setting the option.
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
function single($ch): void {
    curl_setopt($ch, CURLOPT_HTTPHEADER, ['X-A: ' . $_GET['x']]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHeader: Tainted HTTP header — possible header injection or open redirect
}

function several($ch): void {
    curl_setopt_array($ch, [CURLOPT_HTTPHEADER => ['X-A: ' . $_GET['x']]]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHeader: Tainted HTTP header — possible header injection or open redirect
}

function otherOption($ch): void {
    curl_setopt($ch, CURLOPT_POSTFIELDS, $_GET['x']);
}
