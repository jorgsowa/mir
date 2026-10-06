===description===
Every argument can be a `?` placeholder at once (full currying spelled out
explicitly, rather than via the bare `...` rest marker) — each occupies its
own positionally-aligned slot.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.5</phpVersion>
</mir>
===file===
<?php

function add(int $a, int $b): int {
    return $a + $b;
}

$curried = add(?, ?);
//             ^ ParseError: Parse error: 'partial function application' requires PHP 8.6 or higher
//                ^ ParseError: Parse error: 'partial function application' requires PHP 8.6 or higher
