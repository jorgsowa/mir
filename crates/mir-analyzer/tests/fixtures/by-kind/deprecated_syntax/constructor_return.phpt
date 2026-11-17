===description===
From PHP 8.6 returning a value from a constructor or destructor, or making one a generator, is deprecated; a bare return and closures inside are fine.
===config===
<mir>
  <phpVersion>8.6</phpVersion>
</mir>
===file===
<?php
class Connection {
    public function __construct(bool $lazy) {
        if ($lazy) {
            return;
        }
        $make = fn(): int => 1;
        $close = function (): bool {
            return true;
        };
        $make();
        $close();
        return null;
//      ^^^^^^^^^^^^ DeprecatedSyntax: Returning a value from a constructor is deprecated
    }

    public function __destruct() {
        yield 1;
//      ^^^^^^^ DeprecatedSyntax: Making a destructor a Generator is deprecated
    }
}
