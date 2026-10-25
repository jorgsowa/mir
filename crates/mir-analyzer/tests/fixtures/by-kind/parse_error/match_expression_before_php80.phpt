===description===
match requires PHP 8.0.
===config===
<mir>
  <phpVersion>7.4</phpVersion>
</mir>
===file===
<?php
function f(int $x): string {
    return match ($x) { 1 => "a", default => "b" };
//         ^^^^^ ParseError: 'match expressions' requires PHP 8.0 or higher [version-too-low]
}
