===description===
A built-in function deprecated in PHP 8.6 is reported when targeting 8.6.
===config===
<mir>
  <phpVersion>8.6</phpVersion>
</mir>
===file===
<?php
function describe(object $o, mixed $v): string {
    $hash = spl_object_hash($o);
//          ^^^^^^^^^^^^^^^^^^^ DeprecatedCall: Call to deprecated function spl_object_hash
    $isFloat = is_double($v);
//             ^^^^^^^^^^^^^ DeprecatedCall: Call to deprecated function is_double
    return $hash . ($isFloat ? 'f' : '') . strcoll('a', 'b');
//                                         ^^^^^^^^^^^^^^^^^ DeprecatedCall: Call to deprecated function strcoll
}
