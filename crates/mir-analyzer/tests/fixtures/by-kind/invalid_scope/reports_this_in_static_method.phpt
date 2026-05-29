===description===
reports this in static method
===file===
<?php
class Foo {
    public static function bar(): void {
        $this->close();
    }
}
===expect===
InvalidScope@4:9-4:14: $this cannot be used in a static method
