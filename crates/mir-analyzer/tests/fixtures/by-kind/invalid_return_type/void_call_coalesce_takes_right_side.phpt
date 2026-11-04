===description===
A void call evaluates to null, so `void ?? $x` is the right-hand side's type
===file===
<?php
function v(): void {}
function f(): bool {
    return v() ?? true;
}
