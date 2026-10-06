===description===
`new` always constructs immediately, so PHP forbids partial function
application there — the parser rejects a placeholder constructor argument
with its own "Cannot use partial function application in new expression"
error (on top of, or instead of, the ordinary version-gate error). Locks in
whatever mir currently surfaces for it, without crashing.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.5</phpVersion>
</mir>
===file===
<?php

class Point {
    public function __construct(
        public int $x,
        public int $y,
    ) {}
}

$p = new Point(?, 2);
//             ^ ParseError: Parse error: 'partial function application' requires PHP 8.6 or higher
//             ^ ParseError: Parse error: Cannot use partial function application in new expression
