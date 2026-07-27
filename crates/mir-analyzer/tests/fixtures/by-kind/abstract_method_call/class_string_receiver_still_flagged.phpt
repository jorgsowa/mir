===description===
AbstractMethodCall still fires for a class-string receiver ($cls::method()) — unlike an object instance, a class-string variable can hold the literal abstract class name itself, so it gets no "must be concrete" guarantee.
===file===
<?php
abstract class Loader {
    abstract public static function getType(): string;
}
class Consumer {
    public function run(): string {
        $cls = Loader::class;
        return $cls::getType();
    }
}
===expect===
AbstractMethodCall@8:15-8:30: Cannot call abstract method Loader::getType()
