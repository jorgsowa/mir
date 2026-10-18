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
    $upper = $_GET['x'] |> strtoupper(...);
    echo $upper;
}

function notReported(): void {
    echo $_GET['x'] |> htmlspecialchars(...);
    echo $_GET['x'] |> trim(...) |> htmlspecialchars(...);
    echo $_GET['n'] |> intval(...);
    echo 'literal' |> strtoupper(...);
}
===expect===
TaintedHtml@3:4-3:33: Tainted HTML output — possible XSS
TaintedHtml@5:4-5:16: Tainted HTML output — possible XSS
