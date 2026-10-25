===description===
From PHP 8.6 an enum may declare `__debugInfo`.
===config===
<mir>
  <phpVersion>8.6</phpVersion>
</mir>
===file===
<?php
enum Suit {
    case Hearts;

    public function __debugInfo(): array { return []; }
}
