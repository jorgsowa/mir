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
    }
}

function outside(): int {
    S::$a = 5;
    S::$b = 6;
    S::$a++;
    S::$a += 2;
    S::$list[] = 2;
    sort(S::$list);
    unset(S::$list[0]);
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
===expect===
InaccessibleProperty@19:8-19:22: Cannot access property S::$a
InaccessibleProperty@24:4-24:13: Cannot access property S::$a
InaccessibleProperty@25:4-25:13: Cannot access property S::$b
InaccessibleProperty@26:4-26:9: Cannot access property S::$a
InaccessibleProperty@27:4-27:14: Cannot access property S::$a
InaccessibleProperty@28:4-28:18: Cannot access property S::$list
InaccessibleProperty@29:9-29:17: Cannot access property S::$list
InaccessibleProperty@30:10-30:21: Cannot access property S::$list
