===description===
MissingPropertyType does NOT fire for a promoted constructor parameter with
no native type hint when the constructor's own `@param` docblock already
gives it an explicit type.
===file===
<?php
class Point {
    /**
     * @param string $label
     */
    public function __construct(public $label) {}
}
===expect===
