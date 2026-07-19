===description===
class_exists($this->prop)/interface_exists($this->prop) narrows the property
to class-string/interface-string, the property-receiver counterpart of the
existing plain-variable narrowing — method_exists()/property_exists() right
below already have this var/prop split, class_exists() family didn't.
===config===
suppress=UnusedVariable,UnusedParam,MissingPropertyType
===file===
<?php
final class Holder {
    /** @var string */
    public $className;

    public function narrowsClassExists(): void {
        if (class_exists($this->className)) {
            /** @mir-check $this->className is class-string */
            $_ = 1;
        }
    }

    public function narrowsInterfaceExists(): void {
        if (interface_exists($this->className)) {
            /** @mir-check $this->className is interface-string */
            $_ = 1;
        }
    }
}
===expect===
