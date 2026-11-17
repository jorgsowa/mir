===description===
Call forms PHP 8.6 deprecates are reported for older targets too.
===config===
<mir>
  <phpVersion>8.5</phpVersion>
</mir>
===file===
<?php
define('APP_DEBUG', false, false);
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ DeprecatedCall: Call to deprecated function define: define(): Argument #3 ($case_insensitive) is ignored and treated as false since declaration of case-insensitive constants is no longer supported, passing the argument explicitly is unnecessary
define('LET', 1);
//<^^^^^^^^^^^^^^^^ DeprecatedSyntax: Calling a constant "let" is deprecated since 8.6

function check(string $class): bool {
    return is_a($class, Countable::class);
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ DeprecatedCall: Call to deprecated function is_a: Calling is_a() with a string when $allow_string is false
}
