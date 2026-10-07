===description===
`pure-callable`/`pure-Closure` with no `(...)` signature parse as
`pure-callable`/`Closure` instead of a bogus named class — the parenthesized
form (`pure-callable(int): string`) already worked via
`parse_callable_syntax`, but a bare keyword with no signature never reached
it.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param pure-callable $cb */
function useCallable($cb): void {
    /** @mir-check $cb is pure-callable */
    $_ = 1;
}

/** @param pure-Closure $c */
function useClosure($c): void {
    /** @mir-check $c is Closure */
    $_ = 1;
}
