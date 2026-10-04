===description===
An argument whose `Cls::CONST` reference can't be resolved has an unknown value, so it is not reported against a constant-typed parameter.
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
/** @psalm-type Data = array{kind: Kind::A|Kind::B} */
final class Reader {
    /** @param Data $d */
    public function run(array $d): string {
        return self::take($d['kind']);
    }
    /** @param Kind::* $k */
    private static function take(string $k): string {
        return $k;
    }
}
===expect===
