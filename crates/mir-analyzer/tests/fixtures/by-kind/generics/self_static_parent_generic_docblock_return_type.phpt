===description===
`@return self<T>` names the declaring class with the written type
arguments, so a call through a subclass receiver returns `Collection<int>`
instead of collapsing to `mixed`.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template T */
class Collection {
    /** @return self<T> */
    public function identity(): self {
        return $this;
    }
}

class IntCollection extends Collection {}

/** @param IntCollection<int> $c */
function test($c): void {
    $result = $c->identity();
    /** @mir-check $result is Collection<int> */
    $_ = 1;
}
