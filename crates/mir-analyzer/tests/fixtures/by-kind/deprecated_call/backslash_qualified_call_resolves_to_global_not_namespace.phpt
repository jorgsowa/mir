===description===
M13: a root-namespace-qualified call (`\foo()`) inside a namespaced file
must resolve to PHP's real global function, even when a same-named
function exists in the current namespace — the classic deprecated-wrapper
idiom (`namespace App; function json_encode() { return \json_encode(...); }`).
The bare call from another function still resolves to (and flags) the local
namespaced wrapper.
===config===
suppress=UnusedParam,MissingParamType
===file===
<?php
namespace App;

/** @deprecated use PHP's json_encode() instead. */
function json_encode($value): string {
    return \json_encode($value);
}

function useIt($x): void {
    json_encode($x);
}
===expect===
DeprecatedCall@10:4-10:19: Call to deprecated function App\json_encode: use PHP's json_encode() instead.
