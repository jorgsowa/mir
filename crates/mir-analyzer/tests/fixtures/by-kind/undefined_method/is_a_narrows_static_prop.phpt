===description===
`is_a(self::$prop, X::class)` (and `static::$prop`) must narrow the
static-property receiver like the already-correct instance-property and
variable cases — extract_static_prop_access was never checked at this
dispatch site.
===config===
suppress=MissingConstructor,PossiblyNullArgument,MissingPropertyType
===file===
<?php
class Base {}
class Foo extends Base {
    public function fooMethod(): void {}
}

class Container {
    protected static ?Base $item = null;

    public static function useIt(): void {
        if (is_a(self::$item, Foo::class)) {
            self::$item->fooMethod();
        }
    }
}

class ChildContainer extends Container {
    public static function useItViaStatic(): void {
        if (is_a(static::$item, Foo::class)) {
            static::$item->fooMethod();
        }
    }
}

class AllowStringContainer {
    /** @var class-string<Foo>|Base */
    protected static $item;

    public static function useAllowString(): void {
        if (is_a(self::$item, Foo::class, true)) {
            // Both a Foo instance and a "Foo"-family class-string satisfy this —
            // must not be marked unreachable/collapsed just because the string
            // branch alone doesn't narrow to an object.
        }
    }
}
===expect===
