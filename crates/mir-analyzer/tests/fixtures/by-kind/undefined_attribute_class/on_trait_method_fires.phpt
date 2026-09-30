===description===
UndefinedAttributeClass fires when an undefined attribute is placed on a trait method.
===file===
<?php
trait Logging {
    #[Cache]
//    ^^^^^ UndefinedAttributeClass: Attribute class Cache does not exist
    public function log(): void {}
}
===expect===
