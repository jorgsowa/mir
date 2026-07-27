===description===
A broad family of non-sanitizing string-transform builtins (str_replace,
trim, explode/implode, preg_replace, …) never propagated taint at all —
echoing their result with a tainted argument silently produced no
diagnostic, even though none of these functions removes arbitrary
attacker-controlled content. Genuine sanitizers/encoders like
htmlspecialchars are deliberately excluded and stay unflagged.
===config===
suppress=MixedArrayAccess,MixedArgument
===file===
<?php
function viaStrReplace(): void {
    echo str_replace('a', 'b', $_GET['name']);
}

function viaTrim(): void {
    echo trim($_GET['name']);
}

function viaExplodeImplode(): void {
    $parts = explode(',', $_GET['csv']);
    echo implode('-', $parts);
}

function viaPregReplace(): void {
    echo preg_replace('/x/', 'y', $_GET['name']);
}

function viaHtmlspecialchars(): void {
    echo htmlspecialchars($_GET['name']);
}

function staticOnly(): void {
    echo str_replace('a', 'b', 'static');
}
===expect===
TaintedHtml@3:4-3:46: Tainted HTML output — possible XSS
TaintedHtml@7:4-7:29: Tainted HTML output — possible XSS
TaintedHtml@12:4-12:30: Tainted HTML output — possible XSS
TaintedHtml@16:4-16:49: Tainted HTML output — possible XSS
