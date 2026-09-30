===description===
Negative control: a native-only override whose hint cannot hold the parent's docblock-refined
return type is still a mismatch.
===config===
suppress=UnusedParam
===file===
<?php
class Item {}

/** @template T */
abstract class C {
    /** @return class-string<T> */
    abstract public static function k(): string;
}

/** @template-extends C<Item> */
final class D extends C {
    public static function k(): int {
        return 1;
    }
}
===expect===
MethodSignatureMismatch@12:4-12:37: Method D::k() signature mismatch: return type 'int' is not a subtype of parent 'class-string<Item>'
