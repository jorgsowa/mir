===description===
`value-of<Cls::CONST>` / `key-of<Cls::CONST>` project a class-constant array; an unresolvable constant accepts anything.
===file===
<?php
namespace Lib;

final class Kinds {
    public const array MAP = ['x' => 1, 'y' => 2];
    public const array NAMES = ['a', 'b'];
    public const PREFIX_ONE = ['p'];
    public const PREFIX_TWO = ['q'];
    public const int SCALAR = 1;
}

class Repo {
    /** @param list<value-of<Kinds::NAMES>> $names */
    public function names(array $names): void { echo count($names); }

    /** @param value-of<Kinds::MAP> $v */
    public function value(int $v): void { echo $v; }

    /** @param key-of<Kinds::MAP> $k */
    public function key(string $k): void { echo $k; }

    /** @param value-of<Kinds::PREFIX_*> $v */
    public function prefixed(string $v): void { echo $v; }

    /** @param value-of<Kinds::SCALAR> $v */
    public function scalarConstant(string $v): void { echo $v; }

    /** @param value-of<Kinds::MISSING> $v */
    public function missingConstant(string $v): void { echo $v; }

    /** @param value-of<Unknown::NAMES> $v */
    public function missingClass(string $v): void { echo $v; }

    /** @return value-of<Kinds::MAP> */
    public function returned(): int {
        return 2;
    }

    /** @return value-of<Kinds::MAP> */
    public function returnedOutOfRange(): int {
        return 3;
//      ^^^^^^^^^ InvalidReturnType: Return type '3' is not compatible with declared 'value-of<Lib\Kinds::MAP>'
    }

    /**
     * @param value-of<Kinds::MAP> $v
     * @return value-of<Kinds::MAP>
     */
    public function passThrough(int $v): int {
        takesInt($v);
        return $v;
    }
}

function takesInt(int $i): void { echo $i; }

function caller(Repo $r): void {
    $valid = ['a', 'b'];
    /** @mir-check $valid is array{0: "a", 1: "b"} */
    $r->names($valid);

    $invalid = ['zzz'];
    /** @mir-check $invalid is array{0: "zzz"} */
    $r->names($invalid);
//            ^^^^^^^^ InvalidArgument: Argument $names of names() expects 'list<"a"|"b">', got 'array{0: "zzz"}'

    $r->value(2);
    $r->value(3);
//            ^ InvalidArgument: Argument $v of value() expects '1|2', got '3'
    $r->key('x');
    $r->key('zz');
//          ^^^^ InvalidArgument: Argument $k of key() expects '"x"|"y"', got '"zz"'
    $r->prefixed('q');
    $r->prefixed('z');
//               ^^^ InvalidArgument: Argument $v of prefixed() expects '"p"|"q"', got '"z"'
    $r->scalarConstant('anything');
    $r->missingConstant('anything');
    $r->missingClass('anything');
}
===expect===
