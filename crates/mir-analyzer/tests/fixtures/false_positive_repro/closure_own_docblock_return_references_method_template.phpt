===description===
A closure literal's OWN leading `@return T` docblock referencing the
enclosing *method's* own `@template` (not a class-level one, mirrors
`local_var_annotation_references_method_template.phpt`) must also resolve
`T` to a `TTemplateParam` instead of an ordinary (and namespace-mis-
qualified) class reference.
===config===
suppress=UnusedParam
===file===
<?php
namespace App;

class Box {
    /**
     * @template T
     * @param T $value
     * @return Closure(): T
     */
    public function wrap($value) {
        return (
            /** @return T */
            function () use ($value) {
                return $value;
            }
        );
    }
}
===expect===
