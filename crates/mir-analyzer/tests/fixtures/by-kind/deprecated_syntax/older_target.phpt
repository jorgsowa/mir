===description===
PHP 8.6's compile-time deprecations are reported for older targets too.
===config===
<mir>
  <phpVersion>8.5</phpVersion>
</mir>
===file===
<?php
class Let {
//    ^^^ DeprecatedSyntax: Using "let" as a class name is deprecated since 8.6
    const namespace = 'app';
//        ^^^^^^^^^ DeprecatedSyntax: Declaring class constant called 'namespace' is deprecated

    public function __construct() {
        return null;
//      ^^^^^^^^^^^^ DeprecatedSyntax: Returning a value from a constructor is deprecated
    }

    public function __debugInfo(): ?array {
//                                 ^^^^^^ DeprecatedSyntax: Returning null from Let::__debugInfo() is deprecated, make the return type non-nullable and return an empty array instead
        return [];
    }
}

function load(callable $read): mixed {
    try {
        return $read();
    } finally {
        return null;
//      ^^^^^^^^^^^^ DeprecatedSyntax: Returning from a finally block is deprecated
    }
}
