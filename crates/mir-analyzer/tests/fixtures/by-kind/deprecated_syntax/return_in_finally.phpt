===description===
From PHP 8.6 a return inside a finally block is deprecated, also nested in another statement; a closure or class declared there has its own scope.
===config===
<mir>
  <phpVersion>8.6</phpVersion>
</mir>
===file===
<?php
function load(callable $read): mixed {
    try {
        return $read();
    } finally {
        if (rand(0, 1)) {
            return null;
//          ^^^^^^^^^^^^ DeprecatedSyntax: Returning from a finally block is deprecated
        }
        $cleanup = function (): int {
            return 1;
        };
        $cleanup();
    }
}
