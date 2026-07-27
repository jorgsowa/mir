===description===
AbstractMethodCall does NOT fire for a nullable object receiver (?Loader) calling an abstract method — nullability is a separate concern (PossiblyNullMethodCall), not grounds to drop the "must be concrete" LSB guarantee.
===file===
<?php
abstract class Loader {
    abstract public static function getType(): string;
}
function run(?Loader $l): string {
    return $l::getType();
}
===expect===
PossiblyNullMethodCall@6:11-6:13: Cannot call method getType() on possibly null value
