===description===
Taint flows through `|>` like the call it stands for: a sanitizer on the
right clears it, any other callable passes it on.
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
function reported(): void {
    echo $_GET['x'] |> trim(...);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $upper = $_GET['x'] |> strtoupper(...);
    echo $upper;
//  ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function notReported(): void {
    echo $_GET['x'] |> htmlspecialchars(...);
    echo $_GET['x'] |> trim(...) |> htmlspecialchars(...);
    echo $_GET['n'] |> intval(...);
    echo 'literal' |> strtoupper(...);
}
