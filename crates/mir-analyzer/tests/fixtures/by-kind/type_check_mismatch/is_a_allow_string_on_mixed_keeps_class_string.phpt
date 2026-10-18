===description===
`is_a($mixed, X::class, true)` narrows a mixed subject to either an X instance or a class-string of X.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class JsonResource {}

function allow_string(mixed $x): void {
    if (is_a($x, JsonResource::class, true)) {
        /** @mir-check $x is class-string<JsonResource>|JsonResource */
        $_ = $x;
    }
}

function objects_only(mixed $x): void {
    if (is_a($x, JsonResource::class)) {
        /** @mir-check $x is JsonResource */
        $_ = $x;
    }
}

