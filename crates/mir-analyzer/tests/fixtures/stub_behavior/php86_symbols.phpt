===description===
PHP 8.6 adds Time\Duration, gmp_powm_sec(), gmp_prevprime() and ReflectionAttribute's namespace methods, and narrows filter_var_array() to array|false.
===config===
<mir>
  <phpVersion>8.6</phpVersion>
</mir>
===file===
<?php
use Time\Duration;

function describe(ReflectionAttribute $attr, array $input): string {
    $d = Duration::fromSeconds(5)->add(Duration::fromMinutes(1));
    $p = gmp_prevprime(10);
    $q = gmp_powm_sec(2, 3, 5);
    $filtered = filter_var_array($input);
    /** @mir-check $filtered is array<array-key, mixed>|false */
    echo 1;
    /** @mir-check $d->seconds is int */
    echo 1;
    return $attr->getShortName() . $attr->getNamespaceName() . gmp_strval($p) . gmp_strval($q)
        . $d->seconds . ($filtered === false ? 'invalid' : count($filtered));
}
