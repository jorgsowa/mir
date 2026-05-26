===description===
$this::method() inside a class should not error
===file===
<?php
class Foo {
    public static function bar(): string {
        return "hello";
    }

    public function test(): void {
        $this::bar();
    }
}
===expect===
