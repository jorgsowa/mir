===description===
`new` always constructs immediately, so PHP forbids partial function
application there — the parser rejects a placeholder constructor argument
with its own "Cannot use partial function application in new expression"
error (on top of, or instead of, the ordinary version-gate error). Locks in
whatever mir currently surfaces for it, without crashing.
===config===
suppress=UnusedVariable
===file===
<?php

class Point {
    public function __construct(
        public int $x,
        public int $y,
    ) {}
}

$p = new Point(?, 2);
===expect===
ParseError@10:15-10:16: Parse error: 'partial function application' requires PHP 8.6 or higher (targeting PHP 8.5)
ParseError@10:15-10:16: Parse error: Cannot use partial function application in new expression
