===description===
A write to a `private(set)` or `protected(set)` static property outside its scope is inaccessible; in-scope writes and reads are fine.
===config===
<mir><phpVersion>8.5</phpVersion></mir>
===file===
<?php
class S {
    public static private(set) int $a = 1;
    public static protected(set) int $b = 2;
    public static private(set) array $list = [];

    public static function bump(): void {
        self::$a = 5;
        static::$b = 6;
        self::$a++;
        self::$a += 1;
        self::$list[] = 1;
    }
}

class Sub extends S {
    public static function f(): void {
        self::$b = 7;
        parent::$a = 8;
//      ^^^^^^^^^^^^^^ InaccessibleProperty: Cannot access property S::$a
    }
}

function outside(): int {
    S::$a = 5;
//  ^^^^^^^^^ InaccessibleProperty: Cannot access property S::$a
    S::$b = 6;
//  ^^^^^^^^^ InaccessibleProperty: Cannot access property S::$b
    S::$a++;
//  ^^^^^ InaccessibleProperty: Cannot access property S::$a
    S::$a += 2;
//  ^^^^^^^^^^ InaccessibleProperty: Cannot access property S::$a
    S::$list[] = 2;
//  ^^^^^^^^^^^^^^ InaccessibleProperty: Cannot access property S::$list
    sort(S::$list);
//       ^^^^^^^^ InaccessibleProperty: Cannot access property S::$list
    unset(S::$list[0]);
//        ^^^^^^^^^^^ InaccessibleProperty: Cannot access property S::$list
    return S::$a + S::$b;
}
class Bag implements ArrayAccess {
    public function offsetExists(mixed $o): bool { return true; }
    public function offsetGet(mixed $o): mixed { return 1; }
    public function offsetSet(mixed $o, mixed $v): void {}
    public function offsetUnset(mixed $o): void {}
}

class Holder {
    public static private(set) ?Bag $bag = null;
}

function offset_on_object(): void {
    Holder::$bag[] = 1;
    Holder::$bag['k'] = 2;
    unset(Holder::$bag['k']);
}
