===description===
Reverse DNS, headers kept across a redirect and the CGI environment are client data. Other environment variables aren't.
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
function server(): void {
    echo $_SERVER['REMOTE_HOST'];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $_SERVER['REDIRECT_HTTP_X_FOO'];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function environment(): void {
    echo getenv('HTTP_X_FOO');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function clean(): void {
    echo getenv('PATH');
    echo $_SERVER['DOCUMENT_ROOT'];
}
