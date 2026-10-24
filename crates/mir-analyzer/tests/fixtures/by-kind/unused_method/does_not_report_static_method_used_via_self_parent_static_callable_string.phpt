===description===
A `'self::m'`, `'static::m'` or `'parent::m'` callable string names the calling class, so the method is used and the class exists
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
namespace App;
class Base {
    protected static function fromParent(int $x): int { return $x; }
}
final class Child extends Base {
    private static function viaSelf(int $x): int { return $x; }
    private static function viaStatic(int $x): int { return $x; }
    private static function viaCallUserFunc(): int { return 1; }

    public function run(): void {
        print_r(array_map('self::viaSelf', [1]));
        print_r(array_map('static::viaStatic', [1]));
        print_r(array_map('parent::fromParent', [1]));
        echo call_user_func('self::viaCallUserFunc');
        print_r(array_map('self::nope', [1]));
//                        ^^^^^^^^^^^^ UndefinedMethod: Method App\Child::nope() does not exist
    }
}
(new Child())->run();
