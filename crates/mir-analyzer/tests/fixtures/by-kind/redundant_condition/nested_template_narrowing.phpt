===description===
nested instanceof checks with template narrowing
===file===
<?php
class Base {}
class Extended extends Base {
    public string $data;
}
class Other {}

/**
 * @template TValue as Base|Other
 * @param TValue $value
 */
function nestedCheck(Base|Other $value): void {
    if ($value instanceof Base) {
        if ($value instanceof Extended) {
            echo $value->data;
        }
    }
}
===expect===
MissingConstructor@3:0-3:29: Class Extended has uninitialized properties but no constructor
