===description===
A private static method used only through first-class-callable syntax
(`self::helper(...)`) must not be reported unused.
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
class Foo {
    private static function helper(): void {}

    public static function run(): void {
        (self::helper(...))();
    }
}
===expect===
