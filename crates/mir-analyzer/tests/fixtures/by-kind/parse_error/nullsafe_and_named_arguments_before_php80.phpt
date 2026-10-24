===description===
?-> and named arguments require PHP 8.0.
===config===
<mir>
  <phpVersion>7.4</phpVersion>
</mir>
===file===
<?php
function f(?\DateTime $d, string $s): ?string {
    str_pad($s, length: 3);
//              ^^^^^^ ParseError: Parse error: 'named arguments' requires PHP 8.0 or higher [version-too-low]
    return $d?->format("Y");
//           ^^^ ParseError: Parse error: 'nullsafe operator (?->)' requires PHP 8.0 or higher [version-too-low]
}
