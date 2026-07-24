===description===
`foreach ($_GET as $k => $v) { echo $k; }` — the array KEY is
attacker-controlled exactly as much as the value (a reflected-XSS vector
via GET param NAMES), but only the value binding ever called taint_var;
the key binding set its type without ever tainting it.
===config===
suppress=MixedAssignment,UnusedForeachValue
===file===
<?php
function test(): void {
    foreach ($_GET as $k => $v) {
        echo $k;
    }
}
===expect===
TaintedHtml@4:8-4:16: Tainted HTML output — possible XSS
