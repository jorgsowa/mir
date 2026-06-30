===description===
UndefinedAttributeClass fires when an undefined attribute is placed on a trait method.
===file===
<?php
trait Logging {
    #[Cache]
    public function log(): void {}
}
===expect===
UndefinedAttributeClass@3:6-3:11: Attribute class Cache does not exist
