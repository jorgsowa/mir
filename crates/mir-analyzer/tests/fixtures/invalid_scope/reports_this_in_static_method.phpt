===source===
<?php
class Foo {
    public static function bar(): void {
        $this->close();
    }
}
===expect===
InvalidScope: $this cannot be used in a static method
