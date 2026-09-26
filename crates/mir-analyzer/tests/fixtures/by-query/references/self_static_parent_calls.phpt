===description===
`self::`, `static::` and `parent::` calls all reference the declaring method.
===cursor===
references
===file===
<?php
class Base {
    public static function make(): string { return 'b'; }
    public function a(): string { return self::make(); }
    public function b(): string { return static::make(); }
}
final class Child extends Base {
    public function c(): string { return parent::make(); }
}
echo Base::ma<CURSOR>ke();
===expect===
test.php@4:47-4:51
test.php@5:49-5:53
test.php@8:49-8:53
test.php@10:11-10:15
