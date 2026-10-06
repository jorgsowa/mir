===description===
A broad family of non-sanitizing string-transform builtins (str_replace,
trim, explode/implode, preg_replace, …) never propagated taint at all —
echoing their result with a tainted argument silently produced no
diagnostic, even though none of these functions removes arbitrary
attacker-controlled content. Genuine sanitizers/encoders like
htmlspecialchars are deliberately excluded and stay unflagged.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function viaStrReplace(): void {
    echo str_replace('a', 'b', $_GET['name']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function viaTrim(): void {
    echo trim($_GET['name']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function viaExplodeImplode(): void {
    $parts = explode(',', $_GET['csv']);
    echo implode('-', $parts);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function viaPregReplace(): void {
    echo preg_replace('/x/', 'y', $_GET['name']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function viaHtmlspecialchars(): void {
    echo htmlspecialchars($_GET['name']);
}

function staticOnly(): void {
    echo str_replace('a', 'b', 'static');
}
