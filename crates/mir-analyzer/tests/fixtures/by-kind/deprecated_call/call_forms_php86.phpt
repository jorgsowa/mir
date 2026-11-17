===description===
From PHP 8.6 passing a third argument to define(), and calling is_a() with a string, or is_subclass_of() with a string and $allow_string false, are deprecated. A value that may be an object isn't reported.
===config===
<mir>
  <phpVersion>8.6</phpVersion>
</mir>
===file===
<?php
define('APP_DEBUG', false, false);
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ DeprecatedCall: Call to deprecated function define: define(): Argument #3 ($case_insensitive) is ignored and treated as false since declaration of case-insensitive constants is no longer supported, passing the argument explicitly is unnecessary

function check(string $class, object $object, string|object $either): bool {
    return is_a($class, Countable::class)
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ DeprecatedCall: Call to deprecated function is_a: Calling is_a() with a string when $allow_string is false
        || is_subclass_of($class, Countable::class, false)
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ DeprecatedCall: Call to deprecated function is_subclass_of: Calling is_subclass_of() with a string when $allow_string is false
        || is_a($class, Countable::class, true)
        || is_subclass_of($class, Countable::class)
        || is_a($object, Countable::class)
        || is_a($either, Countable::class);
}

define('APP_ENV', 'prod');
