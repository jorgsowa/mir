===description===
`class-string<self|static|parent>` keeps its keyword through namespace
resolution, so the constant lookup is not made on `Ns\self`.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

abstract class Base {
    const ?string SCHEMA = null;

    /** @return class-string<self> */
    public static function selfClass(string $t): string { return static::class; }

    /** @return class-string<static> */
    public static function staticClass(string $t): string { return static::class; }

    public static function viaSelf(string $t) {
        $c = self::selfClass($t);
        /** @mir-check $c is class-string<App\Base> */
        return $c::SCHEMA;
    }

    public static function viaStatic(string $t) {
        $c = static::staticClass($t);
        return $c::SCHEMA;
    }
}

class Child extends Base {
    /** @return class-string<parent> */
    public static function parentClass(): string { return parent::class; }

    public static function viaParent() {
        $c = self::parentClass();
        return $c::SCHEMA;
    }
}
===expect===
