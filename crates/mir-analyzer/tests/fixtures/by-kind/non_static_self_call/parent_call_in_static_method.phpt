===description===
NonStaticSelfCall fires when a non-static parent method is called via parent:: from a static method.
===file===
<?php
class Base {
    public function greet(): string { return "hello"; }
}
class Child extends Base {
    public static function run(): string {
        return parent::greet();
    }
}
===expect===
NonStaticSelfCall@7:15-7:30: Non-static method Base::greet() cannot be called statically
