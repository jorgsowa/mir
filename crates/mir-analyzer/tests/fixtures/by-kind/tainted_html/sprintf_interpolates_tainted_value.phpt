===description===
sprintf()/vsprintf() interpolate every argument straight into the
returned string, but neither was ever modeled as a taint pass-through —
echoing their result with a tainted argument silently produced no
diagnostic at all.
===config===
suppress=MixedArrayAccess,MixedArgument
===file===
<?php
function viaSprintf(): void {
    echo sprintf('<b>%s</b>', $_GET['name']);
}

function viaVsprintf(): void {
    echo vsprintf('<b>%s</b>', [$_GET['name']]);
}

function safeFormatOnly(): void {
    echo sprintf('<b>%s</b>', 'static');
}
===expect===
TaintedHtml@3:4-3:45: Tainted HTML output — possible XSS
TaintedHtml@7:4-7:48: Tainted HTML output — possible XSS
