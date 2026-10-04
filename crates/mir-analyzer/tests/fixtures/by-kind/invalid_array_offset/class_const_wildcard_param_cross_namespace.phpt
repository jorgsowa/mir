===description===
A `@param Other::*` wildcard resolves through `use` imports to the imported class's constants.
===file:Lib/Kind.php===
<?php
namespace Lib;
final class Kind {
    public const string A = 'a';
    public const string B = 'b';
}
===file:App/Reader.php===
<?php
namespace App;
use Lib\Kind;
final class Reader {
    /**
     * @param Kind::* $k
     * @param array<string, int> $map
     */
    public function read(string $k, array $map): int {
        $v = $map[$k];
        /** @mir-check $v is int */
        return $v;
    }
}
===expect===
