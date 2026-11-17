===description===
Before PHP 8.6 the third define() argument and is_a() with a string are not deprecated.
===config===
<mir>
  <phpVersion>8.5</phpVersion>
</mir>
===file===
<?php
define('APP_DEBUG', false, false);
define('LET', 1);

function check(string $class): bool {
    return is_a($class, Countable::class);
}
