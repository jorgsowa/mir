===description===
A by-ref capture of a variable holding only `null` is `mixed` inside the closure, since sibling closures fill it in.
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function sibling(): void {
    $seen = null;
    $set = function (string $id) use (&$seen) { $seen = $id; };
    $chk = function (string $id) use (&$seen) {
        /** @mir-check $seen is mixed */
        return $id === $seen;
    };
    $set('a');
    $chk('a');
}

function nullableUnion(?string $init): void {
    $seen = $init;
    $chk = function (string $id) use (&$seen) {
        /** @mir-check $seen is string|null */
        return $id === $seen;
    };
    $chk('a');
}

function literalWidens(): void {
    $n = 0;
    $f = function () use (&$n) {
        /** @mir-check $n is int */
        return $n === 5;
    };
    $f();
}

function byValueKeepsNull(): void {
    $seen = null;
    $chk = function (string $id) use ($seen) {
        return $id === $seen;
//             ^^^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'string' and 'null' is always false — these types can never be identical
    };
    $chk('a');
}
