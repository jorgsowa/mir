===description===
Enums require PHP 8.1.
===config===
<mir>
  <phpVersion>8.0</phpVersion>
</mir>
===file===
<?php
enum Suit {
//<^^^^ ParseError: Parse error: 'enums' requires PHP 8.1 or higher [version-too-low]
    case Hearts;
}
