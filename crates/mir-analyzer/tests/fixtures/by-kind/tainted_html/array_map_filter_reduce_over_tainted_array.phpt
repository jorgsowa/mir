===description===
array_map()/array_filter()/array_reduce() never propagated taint from
their source array argument — the callback's body isn't analyzed, but the
result can still carry attacker-controlled data through untouched, so
checking the array argument's taint (not the callback) is enough,
mirroring the existing compact()/sprintf() "check the args, not the
callee" shortcut.
===config===
suppress=MixedArrayAccess,MixedArgument,MixedReturnStatement,MixedArgumentTypeCoercion
===file===
<?php
function viaArrayMap(): void {
    echo array_map('strtoupper', $_GET['arr'])[0];
}

function viaArrayFilter(): void {
    echo array_filter($_GET['arr'])[0];
}

function viaArrayReduce(): void {
    echo array_reduce($_GET['arr'], fn($carry, $item) => $carry . $item, '');
}

function staticOnly(): void {
    echo array_map('strtoupper', ['a', 'b'])[0];
}
===expect===
TaintedHtml@3:4-3:50: Tainted HTML output — possible XSS
TaintedHtml@7:4-7:39: Tainted HTML output — possible XSS
TaintedHtml@11:4-11:77: Tainted HTML output — possible XSS
