===description===
From PHP 8.6 a __debugInfo() return type that allows null is deprecated.
===config===
<mir>
  <phpVersion>8.6</phpVersion>
</mir>
===file===
<?php
class Point {
    public function __debugInfo(): ?array {
//                                 ^^^^^^ DeprecatedSyntax: Returning null from Point::__debugInfo() is deprecated, make the return type non-nullable and return an empty array instead
        return [];
    }
}

class Size {
    public function __debugInfo(): array {
        return [];
    }
}
