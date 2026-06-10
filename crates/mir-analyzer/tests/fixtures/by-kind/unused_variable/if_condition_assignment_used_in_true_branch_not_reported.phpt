===description===
Variable assigned in if-condition and used only in the true branch is not reported
===file===
<?php
function getToken(): ?string { return null; }
function useToken(string $t): void {}

if ($token = getToken()) {
    useToken($token);
}

function test(): void {
    $req = null;
    if ($v = rand()) {
        $req = $v;
    }
    echo $req;
}
===expect===
